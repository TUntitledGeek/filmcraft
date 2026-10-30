// Real Chrome disclosure/layout acceptance for the multi-track FilmCraft demo.
export async function runInlineHierarchyChecks({execute,call,send,capture:rawCapture,sleep,report}) {
  const capture=async name=>{await send('Input.dispatchMouseEvent',{type:'mouseMoved',x:2,y:2,button:'none'});await rawCapture(name);};
  const assert=(value,message)=>{if(!value)throw new Error(message);};
  const element=async id=>(await call('ui.elements')).find(e=>e.id===id);
  const geometry=()=>call('ui.timeline.animation');
  const inspect=()=>call('ui.inspect');
  let sequence=await execute('sequence.inspect');
  const clips=()=>sequence.video.flatMap(t=>t.items.map(i=>({...i,track:t.id,kind:'Video'}))).concat(sequence.audio.flatMap(t=>t.items.map(i=>({...i,track:t.id,kind:'Audio'}))));
  const scene=clips().find(i=>i.name==='Desert_Dunes.mp4'&&i.kind==='Video');
  assert(scene,'Hierarchy acceptance requires the representative demo project');
  report.hierarchy={addedEffects:[],snapshots:{}};
  for(const effect of ['lumetri','gaussian_blur'])if(!scene.effects.some(e=>e.effect===effect)){
    await execute('effects.apply',{clips:[scene.clip],effect});report.hierarchy.addedEffects.push(effect);
  }
  sequence=await execute('sequence.inspect');
  const byId=id=>clips().find(i=>i.clip===id);
  const owner=(g,c)=>g.tracks.find(r=>r.track===c.track).rect;
  const ownerVisible=(g,c)=>{const r=owner(g,c),top=c.kind==='Video'?g.content[1]:g.split+5,bottom=c.kind==='Video'?g.split:g.content[1]+g.content[3]-34;return r[1]>=top-.1&&r[1]+r[3]<=bottom+.1;};
  const snapshot=async name=>{const g=await geometry();report.hierarchy.snapshots[name]=g;return g;};
  const drag=async(a,b)=>{
    await send('Input.dispatchMouseEvent',{type:'mouseMoved',...a});
    await send('Input.dispatchMouseEvent',{type:'mousePressed',...a,button:'left',buttons:1,clickCount:1});
    for(let n=1;n<=10;n++){await send('Input.dispatchMouseEvent',{type:'mouseMoved',x:a.x+(b.x-a.x)*n/10,y:a.y+(b.y-a.y)*n/10,button:'left',buttons:1});await sleep(30);}
    await send('Input.dispatchMouseEvent',{type:'mouseReleased',...b,button:'left',clickCount:1});await sleep(300);
  };
  const wheel=async(c,deltaY)=>{const g=await geometry();const top=c.kind==='Video'?g.content[1]:g.split+5,bottom=c.kind==='Video'?g.split:g.content[1]+g.content[3]-34;const point={x:g.content[0]+g.content[2]*.7,y:(top+bottom)/2};await send('Input.dispatchMouseEvent',{type:'mouseMoved',...point,button:'none'});await send('Input.dispatchMouseEvent',{type:'mouseWheel',...point,deltaX:0,deltaY});await sleep(150);};
  const visible=async(e,c)=>{if(!e)return false;const g=await geometry(),y=e.rect[1]+e.rect[3]/2;return y>=(c.kind==='Video'?g.content[1]:g.split+5)+1&&y<(c.kind==='Video'?g.split:g.content[1]+g.content[3]-34)-1;};
  const reveal=async(id,c)=>{
    let e=await element(id);if(await visible(e,c))return e;
    await wheel(c,-10000);
    for(let n=0;n<60;n++){e=await element(id);if(await visible(e,c))return e;await wheel(c,45);}
    report.hierarchy.revealFailure={id,geometry:await geometry(),element:await element(id)};
    throw new Error('Cannot reveal hierarchy control '+id);
  };
  const click=async(id,c)=>{const e=await reveal(id,c);const [x,y,w,h]=e.rect;for(const type of ['mouseMoved','mousePressed','mouseReleased'])await send('Input.dispatchMouseEvent',{type,x:x+w/2,y:y+h/2,button:type==='mouseMoved'?'none':'left',clickCount:1});await sleep(200);};
  const root=id=>`timeline.animation.clip.${id}`;
  const effect=(c,id)=>`${root(c.clip)}.effect.${id}`;
  const param=(c,fx,id)=>`${effect(c,fx)}.param.${id}`;
  const value=async(c,fx,id)=>{const q=await execute('sequence.inspect');return q.video.concat(q.audio).flatMap(t=>t.items).find(i=>i.clip===c.clip).effects.find(e=>e.effect===fx).params[id];};
  const editDrag=async(id,c,dx)=>{const e=await reveal(id,c),[x,y,w,h]=e.rect;await drag({x:x+w/2,y:y+h/2},{x:x+w/2+dx,y:y+h/2});};
  const assertSplit=(before,after)=>assert(Math.abs(before.split-after.split)<.1,'Disclosure unexpectedly moved the video/audio divider');
  const assertDisjoint=g=>{for(let i=0;i<g.drawers.length;i++)for(let j=i+1;j<g.drawers.length;j++){const a=g.drawers[i].visible,b=g.drawers[j].visible;if(!a||!b)continue;assert(Math.min(a[1]+a[3],b[1]+b[3])-Math.max(a[1],b[1])<=.1,'Visible clip hierarchies overlap');}};
  await execute('timeline.select',{clips:[scene.clip],linked:false});await execute('playhead.set',{time:scene.start+254016000000});
  await capture('hierarchy-collapsed-dark');await call('ui.set',{theme:'light'});await capture('hierarchy-collapsed-light');await call('ui.set',{theme:'dark'});
  const closed=await snapshot('factoryCollapsed');
  // Enlarge the dock through its real resize handle for the detailed animation captures.
  const gutter=(await call('ui.elements')).find(e=>e.id.startsWith('dock.gutter.')&&e.rect[2]>600&&e.rect[3]<12);
  assert(gutter,'No main timeline dock resize handle');
  let [gx,gy,gw,gh]=gutter.rect;await drag({x:gx+gw/2,y:gy+gh/2},{x:gx+gw/2,y:gy+gh/2-170});
  const before=await snapshot('largerDockCollapsed');
  await click(`${root(scene.clip)}.toggle`,scene);
  let g=await snapshot('headers');assertSplit(before,g);assert(ownerVisible(g,scene),'Opening animation hid the owning clip');assertDisjoint(g);
  const view=g.view.clips.find(v=>v.clip===scene.clip);assert(view&&view.effects.length===0&&view.properties.length===0,'A newly opened clip expanded its properties automatically');
  assert(g.revision===before.revision,'Opening animation mutated the project');
  await capture('hierarchy-effect-headers-dark');await call('ui.set',{theme:'light'});await capture('hierarchy-effect-headers-light');await call('ui.set',{theme:'dark'});
  const ownerBefore=owner(g,scene)[1];
  await click(`${effect(scene,'motion')}.toggle`,scene);g=await snapshot('motion');assertSplit(before,g);assert(ownerVisible(g,scene),'Opening Motion hid its owning clip');assert(Math.abs(owner(g,scene)[1]-ownerBefore)<.1,'Group expansion moved the owner');
  await capture('hierarchy-motion-dark');await call('ui.set',{theme:'light'});await capture('hierarchy-motion-light');await call('ui.set',{theme:'dark'});
  const position=await value(scene,'motion','position');await editDrag(`${param(scene,'motion','position')}.x`,scene,25);
  const changedPosition=await value(scene,'motion','position');assert(position.value!==changedPosition.value&&changedPosition.keyframes===0,'Typed Position X editing failed or implicitly animated');
  await execute('edit.undo');assert((await value(scene,'motion','position')).value===position.value,'Position undo failed');await execute('edit.redo');assert((await value(scene,'motion','position')).value===changedPosition.value,'Position redo failed');
  report.hierarchy.position={before:position,after:changedPosition,undoRedo:true};
  await click(`${effect(scene,'motion')}.toggle`,scene);
  await click(`${effect(scene,'opacity')}.toggle`,scene);
  const opacity=await value(scene,'opacity','opacity');await editDrag(`${param(scene,'opacity','opacity')}.value`,scene,-25);
  assert((await value(scene,'opacity','opacity')).value!==opacity.value,'Reusable scalar editor did not change Opacity');
  await click(`${param(scene,'opacity','opacity')}.toggle`,scene);
  await click(`${param(scene,'opacity','opacity')}.addKey`,scene);assert((await value(scene,'opacity','opacity')).keyframes===1,'First inline key not created');
  await execute('edit.undo');assert((await value(scene,'opacity','opacity')).keyframes===0,'Key undo failed');await execute('edit.redo');
  await execute('playhead.set',{time:scene.start+508032000000});await click(`${param(scene,'opacity','opacity')}.addKey`,scene);assert((await value(scene,'opacity','opacity')).keyframes===2,'Second inline key not created');
  await execute('playhead.set',{time:scene.start+381024000000});
  await editDrag(`${param(scene,'opacity','opacity')}.value`,scene,-20);
  assert((await value(scene,'opacity','opacity')).keyframes===3,'Editing an animated property did not create its canonical playhead key');
  await execute('edit.undo');assert((await value(scene,'opacity','opacity')).keyframes===2,'Animated edit undo failed');
  await execute('edit.redo');assert((await value(scene,'opacity','opacity')).keyframes===3,'Animated edit redo failed');
  await click(`${param(scene,'opacity','opacity')}.addKey`,scene);assert((await value(scene,'opacity','opacity')).keyframes===2,'Current-key deletion failed');
  await execute('edit.undo');assert((await value(scene,'opacity','opacity')).keyframes===3,'Key deletion undo failed');
  await execute('edit.redo');assert((await value(scene,'opacity','opacity')).keyframes===2,'Key deletion redo failed');
  report.hierarchy.animatedEditAndDeleteUndoRedo=true;
  report.hierarchy.opacityKeys=await value(scene,'opacity','opacity');
  await capture('hierarchy-opacity-keys-dark');await call('ui.set',{theme:'light'});await capture('hierarchy-opacity-keys-light');await call('ui.set',{theme:'dark'});
  const verifyRuler=async()=>{const g=await geometry(),u=(await inspect()).ui;const keys=(await call('ui.elements')).filter(e=>e.id.startsWith(`${param(scene,'opacity','opacity')}.key.`));assert(keys.length>0,'No visible Opacity keys');for(const key of keys){const source=Number(key.id.split('.').at(-1));const time=scene.start+source-scene.sourceIn;const expected=g.content[0]+(time/254016000000-u.timeline.scroll)*u.timeline.pps;assert(Math.abs(expected-(key.rect[0]+key.rect[2]/2))<1.25,'Inline key drifted from the main ruler');}return {keys:keys.length,pps:u.timeline.pps,scroll:u.timeline.scroll};};
  report.hierarchy.rulerBefore=await verifyRuler();await call('ui.set',{timeline:{pps:90,scroll:Math.max(0,scene.start/254016000000-1)}});await sleep(500);report.hierarchy.rulerAfter=await verifyRuler();await capture('hierarchy-ruler-zoom');await call('ui.set',{timeline:{fit:true}});await sleep(500);
  await click(`${effect(scene,'lumetri')}.toggle`,scene);
  await capture('hierarchy-selective-effects-dark');await call('ui.set',{theme:'light'});await capture('hierarchy-selective-effects-light');await call('ui.set',{theme:'dark'});
  const sectionId=`${effect(scene,'lumetri')}.section.Basic Correction`;
  await click(sectionId,scene);
  const exposure=await value(scene,'lumetri','exposure');await editDrag(`${param(scene,'lumetri','exposure')}.value`,scene,20);assert((await value(scene,'lumetri','exposure')).value!==exposure.value,'Grouped Lumetri property editor failed');await execute('edit.undo');assert((await value(scene,'lumetri','exposure')).value===exposure.value,'Lumetri undo failed');
  report.hierarchy.groupedEditor=true;
  await click(sectionId,scene);
  await click(`${effect(scene,'gaussian_blur')}.toggle`,scene);await capture('hierarchy-multiple-effects');
  g=await snapshot('selective');assertSplit(before,g);assertDisjoint(g);
  // Existing editing gestures must continue to work while the hierarchy is open.
  const projectContent=q=>JSON.stringify({video:q.video,audio:q.audio});
  await execute('timeline.select',{clips:[scene.clip],linked:false});await execute('sequence.snap',{on:false});
  const contentBefore=projectContent(await execute('sequence.inspect'));
  let body=await reveal(`timeline.clip.${scene.clip}`,scene);let [bx,by,bw,bh]=body.rect;
  await drag({x:bx+bw-3,y:by+bh/2},{x:bx+bw-23,y:by+bh/2});
  assert(projectContent(await execute('sequence.inspect'))!==contentBefore,'Clip trim was blocked by expanded animation');
  await execute('edit.undo');assert(projectContent(await execute('sequence.inspect'))===contentBefore,'Expanded clip trim undo failed');
  body=await reveal(`timeline.clip.${scene.clip}`,scene);[bx,by,bw,bh]=body.rect;
  await drag({x:bx+bw*.5,y:by+bh/2},{x:bx+bw*.5+18,y:by+bh/2});
  assert(projectContent(await execute('sequence.inspect'))!==contentBefore,'Clip move was blocked by expanded animation');
  await execute('edit.undo');assert(projectContent(await execute('sequence.inspect'))===contentBefore,'Expanded clip move undo failed');
  // Materialize the demo's implicit defaults before comparing explicit targeting state.
  // Re-enabling a target appends it in the existing engine vector; order is not membership.
  await execute('timeline.setTargeting',{track:'V1',targeted:true,sourcePatch:true});
  const targetingBefore=JSON.stringify((await execute('state.inspect')).targeting);
  for(const id of ['timeline.track.V1.target','timeline.track.V1.sourcePatch']){await click(id,scene);assert(JSON.stringify((await execute('state.inspect')).targeting)!==targetingBefore,'Expanded track control did not change');await click(id,scene);assert(JSON.stringify((await execute('state.inspect')).targeting)===targetingBefore,'Expanded track control did not restore');}
  await execute('sequence.snap',{on:true});report.hierarchy.editingWhileExpanded={trimUndo:true,moveUndo:true,trackControls:true};
  const other=clips().find(c=>c.kind==='Video'&&c.track!==scene.track);assert(other,'No second video track');
  await click(`${root(other.clip)}.toggle`,other);g=await snapshot('anotherTrack');assertSplit(before,g);assert(ownerVisible(g,other),'The newly expanded second-track clip is hidden');assert(g.drawers.length>=2,'Second clip replaced the first disclosure');assertDisjoint(g);await capture('hierarchy-another-clip-dark');await call('ui.set',{theme:'light'});await capture('hierarchy-another-clip-light');await call('ui.set',{theme:'dark'});
  const sameTrack=clips().find(c=>c.kind==='Video'&&c.track===scene.track&&c.clip!==scene.clip);await click(`${root(sameTrack.clip)}.toggle`,sameTrack);g=await snapshot('sameTrack');assert(g.drawers.length>=3,'Multiple clips on one track were not kept expanded');assertDisjoint(g);await capture('hierarchy-same-track');
  await send('Emulation.setDeviceMetricsOverride',{width:1280,height:894,deviceScaleFactor:1,mobile:false});await sleep(400);await capture('hierarchy-narrow-dark');await call('ui.set',{theme:'light'});await capture('hierarchy-narrow-light');await call('ui.set',{theme:'dark'});
  await send('Emulation.setDeviceMetricsOverride',{width:1656,height:894,deviceScaleFactor:1,mobile:false});await sleep(300);
  const currentGutter=(await call('ui.elements')).find(e=>e.id.startsWith('dock.gutter.')&&e.rect[2]>600&&e.rect[3]<12);[gx,gy,gw,gh]=currentGutter.rect;
  await drag({x:gx+gw/2,y:gy+gh/2},{x:gx+gw/2,y:gy+gh/2+170});g=await snapshot('shortTimeline');assert(ownerVisible(g,sameTrack),'Shortening the panel hid the visible owner');assertDisjoint(g);
  await capture('hierarchy-short-dark');await call('ui.set',{theme:'light'});await capture('hierarchy-short-light');await call('ui.set',{theme:'dark'});
  const audio=clips().find(c=>c.kind==='Audio');await click(`${root(audio.clip)}.toggle`,audio);g=await snapshot('audio');assertDisjoint(g);assert(ownerVisible(g,audio),'Audio owner is hidden');await capture('hierarchy-audio');
  for(const c of [scene,other,sameTrack,audio]){await execute('timeline.animation.toggle',{clip:c.clip});}
  g=await snapshot('collapsedAgain');assert(g.drawers.length===0,'Clip disclosure did not collapse independently');
  assert(Math.abs(g.split-closed.split)<2,'The stored video/audio split changed without a user divider action');
  await capture('hierarchy-collapsed-again');
  report.hierarchy.complete=true;
}
