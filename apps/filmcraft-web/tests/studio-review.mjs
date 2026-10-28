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
const proc = spawn(chrome, ['--headless=new', `--remote-debugging-port=${port}`, `--user-data-dir=${mkdtempSync(join(tmpdir(), 'fc-review-'))}`, '--no-first-run', '--no-default-browser-check', '--enable-unsafe-webgpu', '--autoplay-policy=no-user-gesture-required', '--window-size=1656,894', 'about:blank'], {stdio:['ignore','ignore','pipe'], windowsHide:true});
const sleep = ms => new Promise(r => setTimeout(r, ms));
let ws, serial = 0;
const pending = new Map();
const report = {url, viewport:[1656,894], hardware:{cpu:cpus()[0]?.model, logicalCpus:cpus().length, memoryGB:totalmem()/2**30}, errors:[], captures:{}};
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
  await execute('file.openDemoProject');
  await call('ui.set',{mode:'edit',workspace:arg('workspace','Editing')});
  await execute('edit.deselectAll');
  await capture('editing-empty');
  report.sequence=await execute('sequence.inspect');
  // App-level frame statistics plus browser frame pacing and long tasks.
  await execute('playhead.set',{seconds:0});
  report.beforePerformance=await call('perf.stats');
  await js(`window.reviewPacing=[]; window.reviewLongTasks=[]; window.reviewQueue=[]; window.reviewRunning=true;
    new PerformanceObserver(l=>reviewLongTasks.push(...l.getEntries().map(e=>({start:e.startTime,duration:e.duration})))).observe({type:'longtask',buffered:false});
    let prev; function sample(t){if(prev!==undefined)reviewPacing.push(t-prev);prev=t;if(reviewRunning)requestAnimationFrame(sample)}requestAnimationFrame(sample);
    void (async function sampleQueue(){while(reviewRunning){const s=await filmcraft.request('perf.stats');reviewQueue.push({at:performance.now(),queued:s.frames.queued,frame:s.monitor.frame,due:s.monitor.playheadFrame});await new Promise(r=>setTimeout(r,100));}})();`);
  await call('ui.playback',{action:'play'});
  await sleep(seconds*1000);
  report.performance=await call('perf.stats');
  report.performance.browser=await js(`(()=>{reviewRunning=false;const a=reviewPacing.slice().sort((a,b)=>a-b);return {durationS:${seconds},samples:a.length,p50Ms:a[Math.floor(a.length*.5)],p95Ms:a[Math.floor(a.length*.95)],maxMs:a.at(-1),stallsOver50Ms:a.filter(x=>x>50).length,longTasks:reviewLongTasks,queueSamples:reviewQueue}})()`);
  await call('ui.playback',{action:'stop'});
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
    if(workspace==='Color' && first) {
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
