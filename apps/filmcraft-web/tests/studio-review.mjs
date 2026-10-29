// Real browser evidence at the review viewport. Uses the app's control API and CDP.
// node .../studio-review.mjs --url URL --out DIR [--seconds 12]
import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync, mkdtempSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir, cpus, totalmem } from 'node:os';
const arg = (n, d) => { const i = process.argv.indexOf(`--${n}`); return i < 0 ? d : process.argv[i + 1]; };
const url = arg('url', 'http://127.0.0.1:8766/');
const out = arg('out', 'target/studio-review');
const seconds = Number(arg('seconds', '12'));
const port = Number(arg('port', String(9400 + Math.floor(Math.random()*500))));
mkdirSync(out, {recursive:true});
const chrome = arg('chrome', 'C:/Program Files/Google/Chrome/Application/chrome.exe');
const softwareVideo=arg('software-video','false')==='true';
const unsafeWebgpu=arg('unsafe-webgpu','false')==='true';
const proc = spawn(chrome, ['--headless=new', `--remote-debugging-port=${port}`, `--user-data-dir=${mkdtempSync(join(tmpdir(), 'fc-review-'))}`, '--no-first-run', '--no-default-browser-check', ...(unsafeWebgpu?['--enable-unsafe-webgpu']:[]), ...(softwareVideo?['--disable-accelerated-video-decode']:[]), '--autoplay-policy=no-user-gesture-required', '--window-size=1656,894', 'about:blank'], {stdio:['ignore','ignore','pipe'], windowsHide:true});
const sleep = ms => new Promise(r => setTimeout(r, ms));
let ws, serial = 0;
const pending = new Map();
const report = {url, viewport:[1656,894], hardware:{cpu:cpus()[0]?.model, logicalCpus:cpus().length, memoryGB:totalmem()/2**30}, errors:[], captures:{}};
report.hardwareVideoDecodeDisabled=softwareVideo;
report.unsafeWebgpu=unsafeWebgpu;
proc.stderr.on('data',s=>{report.chromeLog=((report.chromeLog??'')+s).slice(-12000);});
function send(method, params={}) {
  return new Promise((resolve,reject) => {
    const id=++serial;
    const timeout=setTimeout(()=>{pending.delete(id);reject(new Error(`CDP timeout: ${method}`));},120000);
    pending.set(id, m=>{clearTimeout(timeout);m.error?reject(new Error(m.error.message)):resolve(m.result);});
    ws.send(JSON.stringify({id,method,params}));
  });
}
async function js(expression) {
  report.lastExpression=expression.slice(0,240);
  const r=await send('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true,timeout:110000});
  if(r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description??r.exceptionDetails.text);
  return r.result.value;
}
async function capture(name) {
  await sleep(1400);
  const shot=await send('Page.captureScreenshot',{format:'png'});
  writeFileSync(join(out,`${name}.png`),Buffer.from(shot.data,'base64'));
  report.captures[name]=await js('filmcraft.inspect()');
}
async function call(method,params={}) {return js(`filmcraft.request(${JSON.stringify(method)},${JSON.stringify(params)})`);}
async function execute(command,params={}) {return js(`filmcraft.execute(${JSON.stringify(command)},${JSON.stringify(params)})`);}
try {
  proc.on('error',e=>report.errors.push(String(e)));
  for(let i=0;i<100;i++) {
    try {
      const pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      const page=pages.find(p=>p.type==='page');
      if(page) {ws=new WebSocket(page.webSocketDebuggerUrl);await new Promise((r,j)=>{ws.onopen=r;ws.onerror=j;});break;}
    } catch {}
    await sleep(100);
  }
  if(!ws) throw new Error('Chrome did not start');
  ws.onmessage=e=>{const m=JSON.parse(e.data);if(m.id)pending.get(m.id)?.(m);if(m.method==='Runtime.exceptionThrown')report.errors.push(m.params.exceptionDetails);};
  ws.addEventListener('message',e=>{const m=JSON.parse(e.data);if(m.method==='Runtime.consoleAPICalled' && ['warning','error'].includes(m.params.type)){
    (report.console??=[]).push({type:m.params.type,text:m.params.args.map(a=>a.value??a.description??'').join(' ').slice(0,1600)});
  }});
  await send('Page.enable');await send('Runtime.enable');
  report.browser=await send('Browser.getVersion');
  await send('Emulation.setDeviceMetricsOverride',{width:1656,height:894,deviceScaleFactor:1,mobile:false});
  await send('Page.navigate',{url});
  let ready=false;
  for(let i=0;i<240;i++){
    ready=await js('(async()=>{if(!window.filmcraft)return false;try{await filmcraft.inspect();return true;}catch{return false;}})()');
    if(ready)break;
    await sleep(500);
  }
  report.load=await js('window.filmcraftLoad');
  if(!ready) throw new Error(`App control API did not become ready: ${JSON.stringify(report.load)}`);
  report.info=await js('filmcraft.info()');
  report.adapter=await js('(async()=>{const a=await navigator.gpu?.requestAdapter();if(!a)return null;return {vendor:a.info.vendor,architecture:a.info.architecture,device:a.info.device,description:a.info.description}})()');
  const media=arg('media','none');
  if(media==='none') await execute('file.openDemoProject');
  else {
    await execute('file.newProject',{name:'Controlled imported video'});
    report.import=await js(`filmcraft.importUrl(new URL(${JSON.stringify(media)}, location.href).href)`);
    if(report.import.errors?.length || !report.import.items?.length) throw new Error('Media import failed: '+JSON.stringify(report.import));
    await execute('file.newSequence',{name:'AVC 720p30',fromItems:report.import.items});
  }
  await call('ui.set',{mode:'edit',workspace:arg('workspace','Editing')});
  await execute('edit.deselectAll');
  const addedEffects=arg('effects','none').split(',').filter(e=>e!=='none');
  if(addedEffects.length) {
    const sequence=await execute('sequence.inspect');
    const clip=sequence.video.flatMap(t=>t.items).find(i=>i.clip)?.clip;
    if(!clip)throw new Error('No picture clip to receive benchmark effects');
    report.effectSetup=[];
    for(const effect of addedEffects) {
      await execute('effects.apply',{clips:[clip],effect});
      const params=effect==='gaussian_blur'?{blurriness:8}:effect==='lumetri'?{exposure:.6,contrast:12,saturation:110}:effect==='brightness_contrast'?{brightness:10,contrast:15}:{};
      for(const [param,value] of Object.entries(params))await execute('effects.setParam',{clip,effect,param,value});
      report.effectSetup.push({effect,params});
    }
  }
  await capture('editing-empty');
  report.sequence=await execute('sequence.inspect');
  // App-level frame statistics plus browser frame pacing and long tasks.
  await execute('playhead.set',{seconds:0});
  report.beforePerformance=await call('perf.stats');
  await js(`window.reviewPacing=[]; window.reviewLongTasks=[]; window.reviewQueue=[]; window.reviewRunning=true;
    new PerformanceObserver(l=>reviewLongTasks.push(...l.getEntries().map(e=>({start:e.startTime,duration:e.duration})))).observe({type:'longtask',buffered:false});
    let prev; function sample(t){if(prev!==undefined)reviewPacing.push(t-prev);prev=t;if(reviewRunning)requestAnimationFrame(sample)}requestAnimationFrame(sample);
    void (async function sampleQueue(){while(reviewRunning){const s=await filmcraft.request('perf.stats');reviewQueue.push({at:performance.now(),queued:s.frames.queued,frame:s.monitor.frame,due:s.monitor.playheadFrame});await new Promise(r=>setTimeout(r,100));}})();`);
  const playbackStart=performance.now();
  if(arg('cpu-profile','false')==='true') {await send('Profiler.enable');await send('Profiler.start');}
  await call('ui.playback',{action:'play'});
  await sleep(seconds*1000);
  report.performance=await call('perf.stats');
  report.performance.elapsedWallMs=performance.now()-playbackStart;
  if(arg('cpu-profile','false')==='true') {
    const {profile}=await send('Profiler.stop');
    writeFileSync(join(out,'main-thread.cpuprofile'),JSON.stringify(profile));
    const nodes=new Map(profile.nodes.map(n=>[n.id,n.callFrame]));const costs=new Map();
    for(let i=0;i<(profile.samples?.length??0);i++){const f=nodes.get(profile.samples[i]);const key=(f?.functionName||'(anonymous)')+' @ '+(f?.url??'');costs.set(key,(costs.get(key)??0)+(profile.timeDeltas?.[i]??0)/1000);}
    report.cpuTop=[...costs].sort((a,b)=>b[1]-a[1]).slice(0,30).map(([frame,selfMs])=>({frame,selfMs}));
  }
  report.performance.browser=await js(`(()=>{reviewRunning=false;const a=reviewPacing.slice().sort((a,b)=>a-b);return {durationS:${seconds},samples:a.length,p50Ms:a[Math.floor(a.length*.5)],p95Ms:a[Math.floor(a.length*.95)],maxMs:a.at(-1),stallsOver50Ms:a.filter(x=>x>50).length,longTasks:reviewLongTasks,queueSamples:reviewQueue}})()`);
  const pauseStart=performance.now();
  await call('ui.playback',{action:'stop'});
  report.pauseUpperBoundMs=performance.now()-pauseStart;
  report.afterInfo=await js('filmcraft.info()');
  if(arg('editing-tests','false')==='true') {
    await execute('file.openDemoProject');
    await call('ui.set',{workspace:'Clean Editing',mode:'edit'});
    await call('ui.click',{id:'editing.pinSource'});
    await call('ui.playback',{action:'stop'});
    await sleep(2500);
    await js(`window.reviewClicks=[];for(const name of ['pointerdown','pointerup'])addEventListener(name,e=>reviewClicks.push({name,at:performance.now(),detail:e.detail}),{capture:true});`);
    const elements=()=>call('ui.elements');
    const rect=async id=>{const e=(await elements()).find(e=>e.id===id);if(!e)throw new Error('Missing '+id);return e.rect;};
    const point=async(id,fx=.5,fy=.5)=>{const [x,y,w,h]=await rect(id);return {x:x+w*fx,y:y+h*fy};};
    const click=async(p,count=1)=>{
      await send('Input.dispatchMouseEvent',{type:'mouseMoved',...p});
      for(let i=1;i<=count;i++){
        await send('Input.dispatchMouseEvent',{type:'mousePressed',...p,button:'left',clickCount:i});
        await sleep(35);
        await send('Input.dispatchMouseEvent',{type:'mouseReleased',...p,button:'left',clickCount:i});
        await sleep(45);
      }
      await sleep(300);
    };
    const drag=async(a,b)=>{
      await send('Input.dispatchMouseEvent',{type:'mouseMoved',...a});
      await send('Input.dispatchMouseEvent',{type:'mousePressed',...a,button:'left',clickCount:1});
      for(let i=1;i<=12;i++){
        await send('Input.dispatchMouseEvent',{type:'mouseMoved',x:a.x+(b.x-a.x)*i/12,y:a.y+(b.y-a.y)*i/12,button:'left',buttons:1});
        await sleep(20);
      }
      await send('Input.dispatchMouseEvent',{type:'mouseReleased',...b,button:'left',clickCount:1});
      await sleep(500);
    };
    const assert=(ok,message)=>{if(!ok)throw new Error(message);};
    const content=sequence=>JSON.stringify([sequence.video,sequence.audio]);
    const bin=(await elements()).find(e=>/^project\.bin\.\d+$/.test(e.id)&&e.label==='Footage');
    assert(bin,'Missing Footage bin');
    await click(await point(bin.id),2);
    report.editingProbe={state:await js('filmcraft.inspect()'),clicks:await js('reviewClicks')};
    const cards=(await elements()).filter(e=>/^project\.item\.\d+$/.test(e.id));
    assert(cards.length>=2,'Footage double-click did not open media');
    await click(await point(cards[0].id),2);
    const firstId=Number(cards[0].id.split('.').at(-1));
    report.editing={doubleClick:(await execute('source.inspect')).item===firstId};
    assert(report.editing.doubleClick,'Media double-click did not load Source');
    await execute('timeline.setTargeting',{track:'V1',targeted:true,sourcePatch:true});
    const original=await execute('sequence.inspect');
    await drag(await point(cards[1].id),await point('source.dropTarget'));
    report.editing.projectDrop=(await execute('source.inspect')).item===Number(cards[1].id.split('.').at(-1));
    assert(report.editing.projectDrop,'Project drop did not replace Source');
    const clip=original.video.flatMap(t=>t.items).find(i=>i.startFrame===0);
    assert(clip,'Missing timeline clip');
    await drag(await point('timeline.clip.'+clip.clip,.3),await point('source.dropTarget'));
    const source=await execute('source.inspect');
    report.editing.timelineDrop={item:source.item,playhead:source.playhead,unchanged:content(await execute('sequence.inspect'))===content(original)};
    assert(source.item===clip.item&&source.playhead>0&&report.editing.timelineDrop.unchanged,'Timeline drop changed edits or loaded the wrong Source position');
    await click(await point('timeline.clip.'+clip.clip,.5));
    assert((await js('filmcraft.inspect()')).selection.includes(clip.clip),'Timeline pointer selection failed');
    const r=await rect('timeline.clip.'+clip.clip);
    await drag({x:r[0]+r[2]-2,y:r[1]+r[3]*.6},{x:r[0]+r[2]-24,y:r[1]+r[3]*.6});
    report.editing.trimChanged=content(await execute('sequence.inspect'))!==content(original);
    assert(report.editing.trimChanged,'Pointer trim did not edit clip');
    await execute('edit.undo');
    report.editing.trimUndo=content(await execute('sequence.inspect'))===content(original);
    assert(report.editing.trimUndo,'Trim undo failed');
    await execute('edit.redo');
    report.editing.trimRedo=content(await execute('sequence.inspect'))!==content(original);
    await execute('edit.undo');
    await call('ui.set',{focused:'Timeline'});
    await execute('playhead.set',{seconds:2});
    await call('ui.key',{key:'Ctrl+K'});
    report.editing.splitChanged=content(await execute('sequence.inspect'))!==content(original);
    assert(report.editing.splitChanged,'Split keyboard shortcut failed');
    await call('ui.key',{key:'Ctrl+Z'});
    report.editing.splitUndo=content(await execute('sequence.inspect'))===content(original);
    assert(report.editing.splitUndo,'Keyboard undo failed');
    const moveClip=original.video.flatMap(t=>t.items).find(i=>i.startFrame>0);
    if(moveClip){
      await execute('edit.deselectAll');
      const a=await point('timeline.clip.'+moveClip.clip);
      await drag(a,{x:a.x+38,y:a.y});
      report.editing.moveChanged=content(await execute('sequence.inspect'))!==content(original);
      assert(report.editing.moveChanged,'Pointer move did not edit clip');
      await execute('edit.undo');
      report.editing.moveUndo=content(await execute('sequence.inspect'))===content(original);
      assert(report.editing.moveUndo,'Move undo failed');
    }
    report.editing.trackControls={};
    const targeting=async()=>JSON.stringify((await execute('state.inspect')).targeting);
    const originalTargeting=await targeting();
    for(const suffix of ['target','sourcePatch']){
      await click(await point('timeline.track.V1.'+suffix));
      const changed=(await targeting())!==originalTargeting;
      await click(await point('timeline.track.V1.'+suffix));
      const restored=(await targeting())===originalTargeting;
      report.editing.trackControls[suffix]={changed,restored};
      assert(changed&&restored,'Track control failed: '+suffix);
    }
    report.editing.fusionHidden=!(await elements()).some(e=>e.id==='header.mode.fusion');
    assert(report.editing.fusionHidden,'Fusion still in primary navigation');
    await capture('editing-workflows');
  }
  if(arg('benchmark-only','false')==='true') {
    if(arg('seeks','true')==='true') {
    report.seek=await js(`(async()=>{const results=[];for(const seconds of [1,5,2,8,3]){
      const start=performance.now();await filmcraft.execute('playhead.set',{seconds});const control=performance.now()-start;
      let s;for(let i=0;i<200;i++){s=await filmcraft.request('perf.stats');if(s.monitor.frame===s.monitor.playheadFrame)break;await new Promise(r=>setTimeout(r,10));}
      results.push({seconds,commandUpperBoundMs:control,previewUpperBoundMs:performance.now()-start,exact:s.monitor.frame===s.monitor.playheadFrame,monitor:s.monitor,last:s});
    }return results;})()`);
    report.rapidSeek=await js(`(async()=>{
      const start=performance.now();
      for(const seconds of [1,8,2,7,3])await filmcraft.execute('playhead.set',{seconds});
      const commands=performance.now()-start;let s;
      for(let i=0;i<200;i++){s=await filmcraft.request('perf.stats');if(s.monitor.frame===s.monitor.playheadFrame)break;await new Promise(r=>setTimeout(r,10));}
      return {commandsUpperBoundMs:commands,previewUpperBoundMs:performance.now()-start,exact:s.monitor.frame===s.monitor.playheadFrame,monitor:s.monitor};
    })()`);
    }
    report.afterSeekInfo=await js('filmcraft.info()');
    await capture('imported-video');
  } else {
  await execute('playhead.set',{seconds:3});
  const seq=report.sequence;
  const text=JSON.stringify(seq);writeFileSync(join(out,'sequence.json'),text);
  // Select the first picture through the same command used by the timeline.
  const commands=await js('filmcraft.commands()');
  report.selectionCommands=commands.filter(c=>/select|source/i.test(c.id??'')).map(c=>({id:c.id,params:c.params}));
  const first=seq.video.flatMap(t=>t.items).find(i=>i.clip);
  if(first) {
    await execute('timeline.select',{clips:[first.clip],add:false,toggle:false,linked:false});
    await execute('source.open',{item:first.item});
  }
  await capture('editing-selected');
  for(const workspace of ['Fusion','Color']) {
    if(workspace==='Fusion') {
      try { await call('ui.click',{id:'header.mode.fusion'}); }
      catch(e) { (report.workspaceUnavailable??=[]).push({workspace,reason:String(e).slice(0,160)});continue; }
    }
    else await call('ui.set',{workspace});
    if(workspace==='Color' && first && !addedEffects.includes('lumetri')) {
      await execute('effects.apply',{clips:[first.clip],effect:'lumetri'});
    }
    await capture(workspace.toLowerCase());
    if(workspace==='Color' && first) {
      // Pointer timings are upper bounds: the control round trip includes a UI pass.
      const elements=await call('ui.elements');
      const row=(Array.isArray(elements)?elements:elements.elements??[]).find(e=>e.id==='lumetri.param.temperature');
      if(row) {
        const [x,y,w,h]=row.rect;
        const a=x+112+(w-170)*.5, b=x+112+(w-170)*.75, py=y+h/2;
        const before=await call('perf.stats');
        const priorSequence=await execute('sequence.inspect');
        await js(`window.reviewPointerAt=0;addEventListener('pointermove',()=>{reviewPointerAt=performance.now()},{capture:true});`);
        await send('Input.dispatchMouseEvent',{type:'mouseMoved',x:a,y:py});
        await send('Input.dispatchMouseEvent',{type:'mousePressed',x:a,y:py,button:'left',clickCount:1});
        await send('Input.dispatchMouseEvent',{type:'mouseMoved',x:b,y:py,button:'left',buttons:1});
        report.grading=await js(`(async()=>{
          const start=reviewPointerAt;let control,preview,last;
          for(let i=0;i<120;i++){
            last=await filmcraft.request('perf.stats');
            if(last.monitor.projectRevision>${before.monitor.projectRevision} && control===undefined)control=performance.now()-start;
            if(control!==undefined && last.monitor.revision===last.monitor.projectRevision){preview=performance.now()-start;break;}
            await new Promise(r=>setTimeout(r,8));
          }
          return {pointerToControlUpperBoundMs:control,pointerToPreviewUpperBoundMs:preview,during:last};
        })()`);
        await send('Input.dispatchMouseEvent',{type:'mouseReleased',x:b,y:py,button:'left',clickCount:1});
        await sleep(1000);
        report.grading.afterRelease=await call('perf.stats');
        report.grading.sequence=await execute('sequence.inspect');
        const temperature=s=>s.video.flatMap(t=>t.items).find(i=>i.clip===first.clip)?.effects.find(e=>e.effect==='lumetri')?.params.temperature?.value;
        report.grading.valueBefore=temperature(priorSequence);
        report.grading.valueAfter=temperature(report.grading.sequence);
        await execute('edit.undo');
        report.grading.undoRestored=temperature(await execute('sequence.inspect'))===report.grading.valueBefore;
        await execute('edit.redo');
      } else report.grading={limitation:'Temperature slider has no automation rectangle in this build; no pointer latency asserted.'};
    }
  }
  await call('ui.set',{mode:'import'});await capture('project-hub');
  await call('ui.set',{mode:'edit',workspace:arg('workspace','Editing'),theme:'light'});await capture('editing-light');
  if(arg('audit','none')==='all') {
    for(const theme of ['dark','light']) {
      for(const workspace of ['Clean Editing','Fusion','Color','Assembly','Audio','Captions and Graphics']) {
        await call('ui.set',{mode:'edit',workspace,theme});
        await capture(`${workspace.toLowerCase().replaceAll(' ','-')}-${theme}`);
      }
      for(const mode of ['import','export']) {
        await call('ui.set',{mode,theme});await capture(`${mode}-${theme}`);
      }
    }
    for(const width of [1280,1024]) {
      await send('Emulation.setDeviceMetricsOverride',{width,height:894,deviceScaleFactor:1,mobile:false});
      await call('ui.set',{mode:'edit',workspace:'Clean Editing',theme:'dark'});
      if(first) {
        await execute('timeline.select',{clips:[first.clip],add:false,toggle:false,linked:false});
        await execute('source.open',{item:first.item});
      }
      await capture(`editing-dual-${width}`);
      await execute('edit.deselectAll');
      await call('ui.click',{id:'editing.singleProgram'});
      await capture(`editing-single-${width}`);
    }
    await send('Emulation.setDeviceMetricsOverride',{width:1656,height:894,deviceScaleFactor:1,mobile:false});
    await call('ui.set',{mode:'edit',workspace:'Clean Editing',theme:'light'});
  }
  if(arg('workspace','Editing')==='Clean Editing') {
    const before=await js('filmcraft.inspect()');
    const dock=JSON.stringify(before.ui.dock);
    await call('ui.menu.invoke',{id:'window.workspace.saveAs',params:{name:'Studio Review'}});
    await call('ui.set',{workspace:'Classic Editing'});
    await call('ui.set',{workspace:'Studio Review'});
    report.persistence={switchRestored:JSON.stringify((await js('filmcraft.inspect()')).ui.dock)===dock};
    await send('Page.navigate',{url});
    for(let i=0;i<120;i++) {
      const ready=await js('(async()=>{if(!window.filmcraft)return false;try{const s=await filmcraft.inspect();return s.ui.workspace===\"Studio Review\";}catch{return false;}})()');
      if(ready)break;await sleep(100);
    }
    const restored=await js('filmcraft.inspect()');
    report.persistence.reloadRestored=restored.ui.workspace==='Studio Review' && JSON.stringify(restored.ui.dock)===dock;
    if(!report.persistence.switchRestored||!report.persistence.reloadRestored)throw new Error('Custom workspace persistence failed: '+JSON.stringify(report.persistence));
  }
  }
  console.log(JSON.stringify({out,info:report.info,playback:report.performance.playback,pacing:{p50:report.performance.browser.p50Ms,p95:report.performance.browser.p95Ms,stalls:report.performance.browser.stallsOver50Ms},errors:report.errors}));
} catch(e) {
  report.error=String(e);console.error(e);process.exitCode=1;
  if(ws) {
    report.page=await js('({url:location.href,title:document.title,text:document.body.innerText.slice(0,2000),hasApi:!!window.filmcraft})').catch(()=>null);
    const shot=await send('Page.captureScreenshot',{format:'png'}).catch(()=>null);
    if(shot)writeFileSync(join(out,'unavailable.png'),Buffer.from(shot.data,'base64'));
  }
}
finally {writeFileSync(join(out,'report.json'),JSON.stringify(report,null,2));if(ws) await send('Browser.close').catch(()=>{});ws?.close();proc.kill();}
