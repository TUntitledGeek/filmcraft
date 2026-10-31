// Controlled ordinary-Chrome responsiveness/production benchmark. Run after all
// compilers stop; use matched URLs, workloads, fixtures, viewport and quality.
// Fixtures stay local. --project optionally loads a private local .fcproj copy;
// its contents are never included in this report or sent to the web server.
import {spawn} from 'node:child_process';
import {readFileSync,writeFileSync,mkdirSync,mkdtempSync,rmSync} from 'node:fs';
import {join,resolve,sep,basename} from 'node:path';
import {tmpdir,cpus,totalmem} from 'node:os';
import {createHash} from 'node:crypto';
const arg=(key,fallback)=>{const i=process.argv.indexOf('--'+key);return i<0?fallback:process.argv[i+1]};
const url=arg('url','http://127.0.0.1:8783/?fresh&norecover');
const out=arg('out','target/stabilization/browser');mkdirSync(out,{recursive:true});
const workload=arg('workload','demo'),effects=arg('effects','none'),seconds=Number(arg('seconds','6'));
const port=9700+Math.floor(Math.random()*800);
const profile=mkdtempSync(join(tmpdir(),'fc-stabilize-'));
const proc=spawn('C:/Program Files/Google/Chrome/Application/chrome.exe',['--headless=new',`--remote-debugging-port=${port}`,`--user-data-dir=${profile}`,
 '--no-first-run','--no-default-browser-check','--disable-extensions','--disable-background-networking','--autoplay-policy=no-user-gesture-required','--window-size=1656,894','about:blank'],{windowsHide:true,stdio:['ignore','ignore','pipe']});
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
const report={at:new Date().toISOString(),url,workload,effects,seconds,viewport:[1656,894],hardware:{cpu:cpus()[0]?.model,logicalCpus:cpus().length,memoryGB:totalmem()/2**30},errors:[],checks:{}};
if(arg('dist',''))report.wasmSha256=createHash('sha256').update(readFileSync(join(arg('dist',''),'filmcraft_web_bg.wasm'))).digest('hex');
let ws,serial=0;const pending=new Map();
proc.stderr.on('data',b=>report.chromeLog=((report.chromeLog??'')+b).slice(-2500));
const send=(method,params={})=>new Promise((resolve,reject)=>{const id=++serial;const t=setTimeout(()=>{pending.delete(id);reject(Error('CDP timeout '+method))},90000);pending.set(id,m=>{pending.delete(id);clearTimeout(t);m.error?reject(Error(m.error.message)):resolve(m.result)});ws.send(JSON.stringify({id,method,params}))});
const js=async expression=>{const r=await send('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true,userGesture:true,timeout:85000});if(r.exceptionDetails)throw Error(r.exceptionDetails.exception?.description??r.exceptionDetails.text);return r.result.value};
const call=(m,p={})=>js(`filmcraft.request(${JSON.stringify(m)},${JSON.stringify(p)})`);
const execute=(m,p={})=>js(`filmcraft.execute(${JSON.stringify(m)},${JSON.stringify(p)})`);
const assert=(condition,message)=>{if(!condition)throw Error(message)};
const stats=()=>call('perf.stats');
async function picture(frame){const started=performance.now();let s;do{s=await stats();if(s.monitor.frame===frame&&s.monitor.revision===s.monitor.projectRevision)return s;await sleep(10)}while(performance.now()-started<30000);throw Error('Picture timeout '+JSON.stringify(s?.monitor))}
async function capture(name){const s=await send('Page.captureScreenshot',{format:'png'});writeFileSync(join(out,name+'.png'),Buffer.from(s.data,'base64'))}
async function element(id){return(await call('ui.elements')).find(e=>e.id===id)}
const point=e=>({x:Math.min(1535,Math.max(220,e.rect[0]+e.rect[2]/2)),y:e.rect[1]+e.rect[3]/2});
async function mouse(type,p,down=false){return send('Input.dispatchMouseEvent',{type,...p,button:type==='mouseMoved'&&!down?'none':'left',buttons:down?1:0,clickCount:1})}
async function click(p){await mouse('mouseMoved',p);await mouse('mousePressed',p,true);await mouse('mouseReleased',p)}
async function drag(a,b){await mouse('mouseMoved',a);await mouse('mousePressed',a,true);for(let i=1;i<=10;i++){await mouse('mouseMoved',{x:a.x+(b.x-a.x)*i/10,y:a.y+(b.y-a.y)*i/10},true);await sleep(16)}await mouse('mouseReleased',b)}
async function observedCommand(command,params={}){const start=performance.now();await execute(command,params);const acknowledged=performance.now();await js('new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)))');return {ackMs:acknowledged-start,visibleUiUpperBoundMs:performance.now()-start}}
async function playback(label,startFrame,duration){
 await execute('playhead.set',{frame:startFrame});await picture(startFrame);
 const before=await stats();
 await js(`(()=>{const probe=window.fcProbe={pacing:[],longTasks:[],samples:[],running:true};let last;probe.obs=new PerformanceObserver(l=>probe.longTasks.push(...l.getEntries().map(e=>({start:e.startTime,duration:e.duration}))));probe.obs.observe({type:'longtask'});function raf(t){if(last!==undefined)probe.pacing.push(t-last);last=t;if(probe.running)requestAnimationFrame(raf)}requestAnimationFrame(raf);void(async()=>{while(probe.running){const s=await filmcraft.request('perf.stats');probe.samples.push({at:performance.now(),monitor:s.monitor,queue:s.frames.queued,cache:s.frames.cache,audio:filmcraft.info().audioOutput,worker:filmcraft.info().frameWorker});await new Promise(r=>setTimeout(r,100))}})()})()`);
 const started=performance.now();await call('ui.playback',{action:'play'});await sleep(duration*1000);
 const after=await stats(),info=await js('filmcraft.info()');
 const browser=await js(`(()=>{fcProbe.running=false;fcProbe.obs.disconnect();const p=fcProbe.pacing.slice().sort((a,b)=>a-b);return {...fcProbe,obs:undefined,p50Ms:p[Math.floor(p.length*.5)],p95Ms:p[Math.floor(p.length*.95)],maxMs:p.at(-1),stallsOver50:p.filter(x=>x>50).length,stallsOver100:p.filter(x=>x>100).length}})()`);
 const pauseStart=performance.now();await call('ui.playback',{action:'stop'});const pauseMs=performance.now()-pauseStart;
 const result={label,startFrame,elapsedWallMs:performance.now()-started,before,after,info,browser,pauseMs};
 assert(after.monitor.playheadFrame>startFrame||browser.samples.some(s=>s.monitor.playheadFrame!==startFrame),'Playback clock did not advance: '+JSON.stringify(info.audioOutput??after.playback));
 assert(after.playback.presented>1,'No new sequence pictures were actually presented during playback');
 return result;
}
try{
 for(let i=0;i<120;i++){try{const p=(await(await fetch(`http://127.0.0.1:${port}/json/list`)).json()).find(p=>p.type==='page'&&p.url==='about:blank');if(p){ws=new WebSocket(p.webSocketDebuggerUrl);await new Promise((r,j)=>{ws.onopen=r;ws.onerror=j});break}}catch{}await sleep(100)}assert(ws,'Chrome did not start');
 ws.onmessage=e=>{const m=JSON.parse(e.data);if(m.id)pending.get(m.id)?.(m);else if(m.method==='Runtime.exceptionThrown')report.errors.push(m.params.exceptionDetails)};
 await send('Runtime.enable');await send('Page.enable');report.browser=await send('Browser.getVersion');await send('Emulation.setDeviceMetricsOverride',{width:1656,height:894,deviceScaleFactor:1,mobile:false});
 const nav=await send('Page.navigate',{url});assert(!nav.errorText,nav.errorText);
 let ready=false;for(let i=0;i<200;i++){ready=await js('(async()=>{if(!window.filmcraft)return false;try{await filmcraft.inspect();return true}catch{return false}})()');if(ready)break;await sleep(200)}assert(ready,'FilmCraft did not start');
 report.adapter=await js('(async()=>{const a=await navigator.gpu?.requestAdapter();return a?{vendor:a.info.vendor,architecture:a.info.architecture,description:a.info.description}:null})()');
 if(arg('project','')){
  const bytes=readFileSync(arg('project',''));report.projectSha256=createHash('sha256').update(bytes).digest('hex');
  await js(`filmcraft.openProject(new File([${JSON.stringify(bytes.toString('utf8'))}],'stabilization.fcproj',{type:'application/json'}))`);
 }else if(workload==='demo'||workload==='multitrack')await execute('file.openDemoProject');
 else{
  await execute('file.newProject',{name:'Stabilization fixture'});
  const sources=[];const count=workload==='large'?14:1;
  for(let i=0;i<count;i++){const imported=await js(`filmcraft.importUrl(new URL(${JSON.stringify(arg('media','/fixtures/avc720p30.mp4'))},location.href).href)`);assert(!imported.errors?.length&&imported.items?.length,'Fixture import failed '+JSON.stringify(imported));sources.push(...imported.items)}
  await execute('file.newSequence',{name:'Matched imported footage',fromItems:workload==='large'?Array.from({length:72},()=>sources).flat():sources});
 }
 await call('ui.set',{mode:'edit',workspace:'Clean Editing',theme:'dark',playbackRes:'Half',program:{zoom:.5},timeline:{pps:60,scroll:0}});
 await call('ui.playback',{action:'stop'});
 let sequence=await execute('sequence.inspect');const clip=sequence.video.flatMap(t=>t.items).find(c=>c.clip)?.clip;assert(clip,'No picture clip');report.clip=clip;
 report.sequence={frameRate:sequence.settings.frame_rate,videoTracks:sequence.video.length,audioTracks:sequence.audio.length,clips:sequence.video.flatMap(t=>t.items).length};
 if(arg('loop','false')==='true'){await execute('playback.loop',{monitor:'program'});assert((await call('ui.inspect')).playback.loop,'Loop mode was not enabled')}
 if(effects!=='none'){
  for(const effect of (effects==='grade'?['lumetri','gaussian_blur']:['lumetri','gaussian_blur','wave_warp','spherize'])){
   await execute('effects.apply',{clips:[clip],effect});
   const params=effect==='lumetri'?{exposure:.6,contrast:12,saturation:110}:effect==='gaussian_blur'?{blurriness:8}:effect==='wave_warp'?{height:12,wavelength:120}: {radius:160};
   const state=await execute('sequence.inspect');const instance=state.video.flatMap(t=>t.items).find(c=>c.clip===clip).effects.filter(e=>e.effect===effect).at(-1).instanceId;
   for(const [param,value]of Object.entries(params))await execute('effects.setParam',{clip,effect:{instanceId:instance},param,value});
  }
 }
 const initialPictureStart=performance.now();await execute('playhead.set',{frame:15});await picture(15);report.initialPictureMs=performance.now()-initialPictureStart;report.initial=await stats();report.info=await js('filmcraft.info()');assert(!report.info.frameWorkerError,report.info.frameWorkerError);assert(report.info.backend==='BrowserWebGpu','Matched benchmark requires the normal WebGPU backend');
 const stack=(await execute('sequence.inspect')).video.flatMap(t=>t.items).find(c=>c.clip===clip).effects;report.effectIdentity=stack.map(e=>({effect:e.effect,id:e.instanceId,enabled:e.enabled}));
 await capture('before');
 const duration=arg('project','')?Math.min(seconds,2.2):seconds;
 report.playback=[await playback('cold',15,duration),await playback('warm',15,duration)];
 // Cold seeks give the worker new work while ordinary real pointer input is tested.
 report.interactions=[];
 const other=(await execute('sequence.inspect')).video.flatMap(t=>t.items).find(c=>c.clip!==clip)?.clip;
 for(let i=0;i<8;i++){
  await execute('edit.deselectAll');
  const target=other&&i%2===0?other:clip;
  let e=await element('timeline.clip.'+target);if(e&&e.rect[0]>1535)e=await element('timeline.clip.'+clip);assert(e,'Missing timeline clip hit target');
  // Avoid classifying repeated single selections as an intentional double-click
  // (which opens Source). Do not await the seek before injecting input: that would
  // hide a cooperative main-thread stall inside the seek acknowledgement.
  await sleep(650);
  const seek=execute('playhead.set',{frame:25+i*7}).then(value=>({value}),error=>({error:String(error)}));
  await sleep(5);
  const started=performance.now();await click(point(e));
  let selected=false;for(let j=0;j<100;j++){const s=await js('filmcraft.inspect()');if(s.selection.includes(Number(e.id.split('.').at(-1)))){selected=true;break}await sleep(5)}assert(selected,'Real pointer clip selection failed');
  await js('new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)))');
  report.interactions.push({kind:'pointerSelection',underColdSeek:true,ms:performance.now()-started});
  const result=await seek;assert(!result.error,result.error);
 }
 await call('ui.playback',{action:'stop'});await execute('playhead.set',{frame:30});await picture(30);
 const original=JSON.stringify((await execute('sequence.inspect')).video);
 const edge=await element('timeline.clip.'+clip);const [x,y,w,h]=edge.rect;
 await execute('playhead.set',{frame:93});const trimStart=performance.now();await drag({x:x+w-2,y:y+h*.65},{x:x+w-24,y:y+h*.65});
 const trimmed=JSON.stringify((await execute('sequence.inspect')).video);assert(trimmed!==original,'Actual clip-edge drag did not trim');
 report.interactions.push({kind:'trimGesture',gestureAndAckMs:performance.now()-trimStart});
 report.interactions.push({kind:'undoTrim',...await observedCommand('edit.undo')});
 assert(JSON.stringify((await execute('sequence.inspect')).video)===original,'Undo did not restore exact clip/keyframe/effect state');
 // Move the selected clip body and cancel via Escape: leaves canonical data intact.
 const body=await element('timeline.clip.'+clip),a=point(body);await mouse('mouseMoved',a);await mouse('mousePressed',a,true);await mouse('mouseMoved',{x:a.x+35,y:a.y},true);
 for(const type of ['keyDown','keyUp'])await send('Input.dispatchKeyEvent',{type,key:'Escape',code:'Escape',windowsVirtualKeyCode:27});await mouse('mouseReleased',{x:a.x+35,y:a.y});
 await call('ui.inspect');assert(JSON.stringify((await execute('sequence.inspect')).video)===original,'Escape left a partial clip movement');
 report.checks.moveCancel=true;
 // Parameter edits and undo use the existing canonical effect instance IDs.
 const grade=stack.find(e=>e.effect==='lumetri');
 if(grade){report.parameterEdits=[];for(let i=0;i<6;i++){
  await execute('playhead.set',{frame:35+i*5});
  const started=performance.now();await execute('effects.setParam',{clip,effect:{instanceId:grade.instanceId},param:'exposure',value:.7+i*.05});const ack=performance.now()-started;
  await js('new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)))');const uiMs=performance.now()-started;await picture(35+i*5);
  report.parameterEdits.push({ackMs:ack,visibleUiUpperBoundMs:uiMs,pictureMs:performance.now()-started});await execute('edit.undo');
 }}
 assert(JSON.stringify((await execute('sequence.inspect')).video)===original,'Parameter undo changed canonical effects/keyframes');
 if(grade){
  await sleep(650);await click(point(await element('timeline.clip.'+clip)));await call('ui.set',{mode:'color',workspace:'Color'});await sleep(100);
  const slider=await element('lumetri.param.exposure');
  assert(slider,'The active Lumetri Exposure row is not reachable');
  const baseline=JSON.stringify((await execute('sequence.inspect')).video);
  await execute('playhead.set',{frame:104});const [sx,sy,sw,sh]=slider.rect;
  const started=performance.now();await drag({x:sx+sw*.4,y:sy+sh/2},{x:sx+sw*.55,y:sy+sh/2});
  const edited=JSON.stringify((await execute('sequence.inspect')).video);
  assert(edited!==baseline,'Real Lumetri slider drag did not edit the canonical parameter');
  report.lumetriDrag={gestureAndAckMs:performance.now()-started,rect:slider.rect,canonicalChanged:true};
  await execute('edit.undo');assert(JSON.stringify((await execute('sequence.inspect')).video)===baseline,'Slider drag undo failed to restore canonical parameters');
 }
 report.final=await stats();report.finalInfo=await js('filmcraft.info()');await capture('after');assert(!report.finalInfo.frameWorkerError,report.finalInfo.frameWorkerError);assert(!report.errors.length,'Browser runtime exception');report.status='passed';
 console.log(JSON.stringify({out,status:report.status,workload,effects,cold:report.playback[0].after.playback,warm:report.playback[1].after.playback,worker:report.finalInfo.frameWorkers}));
}catch(e){report.error=String(e.stack??e);process.exitCode=1;if(ws){report.finalInfo=await js('window.filmcraft?.info()').catch(()=>null);await capture('failure').catch(()=>{})}console.error(e)}
finally{if(ws)await send('Browser.close').catch(()=>{});ws?.close();proc.kill();if(proc.exitCode===null)await Promise.race([new Promise(r=>proc.once('exit',r)),sleep(2500)]);try{const root=resolve(tmpdir());const owned=resolve(profile);if(!owned.startsWith(root+sep)||!basename(owned).startsWith('fc-stabilize-'))throw Error('Unexpected Chrome profile path');rmSync(owned,{recursive:true,force:true,maxRetries:3,retryDelay:100});report.profileCleanup='removed owned temporary profile'}catch{report.profileCleanup='deferred: Chrome still held its temporary profile'}writeFileSync(join(out,'report.json'),JSON.stringify(report,null,2))}
