// Actual Chrome/CDP editing acceptance. Setup uses canonical commands; all tested gestures
// below are real mouse/key events, with an acknowledged egui frame after each event.
export async function runSharedAnimationChecks({execute,call,send,capture,sleep,report}) {
  const assert=(v,m)=>{if(!v)throw new Error(m);};
  const inspect=()=>call('ui.inspect');
  const elements=()=>call('ui.elements');
  const geometry=()=>call('ui.timeline.animation');
  const frames=async()=>{await inspect();await sleep(35);await inspect();};
  const state=async()=> (await inspect()).ui.keyframe_selection;
  const eq=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
  const element=async id=>(await elements()).find(e=>e.id===id);
  const point=e=>({x:e.rect[0]+e.rect[2]/2,y:e.rect[1]+e.rect[3]/2});
  const mouse=async(type,p,modifiers=0,button='left')=>{await send('Input.dispatchMouseEvent',{type,...p,button:type==='mouseMoved'?'none':button,buttons:type==='mousePressed'?button==='right'?2:1:0,clickCount:1,modifiers});await frames();};
  const clickPoint=async(p,modifiers=0)=>{await mouse('mouseMoved',p,modifiers);await mouse('mousePressed',p,modifiers);await mouse('mouseReleased',p,modifiers);};
  const key=async(name,code,virtual)=>{for(const type of ['keyDown','keyUp']){await send('Input.dispatchKeyEvent',{type,key:name,code,windowsVirtualKeyCode:virtual});await frames();}};
  const wheel=async(video,deltaY)=>{const g=await geometry(),top=video?g.content[1]:g.split+6,bottom=video?g.split:g.content[1]+g.content[3]-34;const p={x:g.content[0]+g.content[2]*.75,y:(top+bottom)/2};await send('Input.dispatchMouseEvent',{type:'mouseMoved',...p,button:'none',buttons:0,modifiers:0});await send('Input.dispatchMouseEvent',{type:'mouseWheel',...p,deltaX:0,deltaY,modifiers:0});await frames();};
  const reveal=async(id,video=true)=>{
    await frames();
    for(let pass=0;pass<2;pass++){
      if(pass)await wheel(video,-10000);
      for(let n=0;n<65;n++){
        const e=await element(id),g=await geometry();
        if(e){const y=point(e).y;if(y>(video?g.content[1]:g.split+6)+1&&y<(video?g.split:g.content[1]+g.content[3]-34)-1)return e;}
        await wheel(video,32);
      }
    }
    report.sharedAnimation.revealFailure={id,geometry:await geometry(),element:await element(id),elements:(await elements()).filter(e=>e.id.startsWith('timeline.animation.clip.')).slice(0,30)};
    throw new Error('Cannot reveal '+id);
  };
  const click=async(id,modifiers=0)=>clickPoint(point(await reveal(id)),modifiers);
  const beginDrag=async(a,b,modifiers=0)=>{
    await mouse('mouseMoved',a,modifiers);await mouse('mousePressed',a,modifiers);
    for(let n=1;n<=10;n++){await send('Input.dispatchMouseEvent',{type:'mouseMoved',x:a.x+(b.x-a.x)*n/10,y:a.y+(b.y-a.y)*n/10,button:'left',buttons:1,modifiers});await frames();}
  };
  const release=async(p,modifiers=0)=>mouse('mouseReleased',p,modifiers);
  const context=async id=>{const p=point(await reveal(id));await mouse('mouseMoved',p);await mouse('mousePressed',p,0,'right');await mouse('mouseReleased',p,0,'right');};
  const menu=async id=>{const e=await element(id);assert(e&&e.enabled!==false,'Unavailable '+id);await clickPoint(point(e));};
  report.sharedAnimation={status:'running',checks:[],domains:'Forward speed=1, non-held effect keys; unsupported domains remain read-only'};
  const checks=report.sharedAnimation.checks;
  await execute('file.openDemoProject');await call('ui.set',{workspace:'Clean Editing',mode:'edit'});await call('ui.playback',{action:'stop'});await frames();
  const gutter=(await elements()).find(e=>e.id.startsWith('dock.gutter.')&&e.rect[2]>600&&e.rect[3]<12);
  assert(gutter,'Missing timeline dock divider for the controlled editing fixture');
  const gutterStart=point(gutter),gutterEnd={x:gutterStart.x,y:gutterStart.y-170};
  await beginDrag(gutterStart,gutterEnd);await release(gutterEnd);
  const q=await execute('sequence.inspect');const frameTicks=Math.round(254016000000*q.settings.frame_rate.den/q.settings.frame_rate.num);report.sharedAnimation.frameRate=q.settings.frame_rate;const c=q.video[0].items[0],other=q.video[1].items[0];
  const root=`timeline.animation.clip.${c.clip}`;
  const effect=async id=>execute('effects.inspect',{clip:c.clip,effect:{instanceId:id}});
  const set=(id,param,value)=>execute('effects.setParam',{clip:c.clip,effect:{instanceId:id},param,value});
  const disclose=async args=>{await execute('timeline.animation.toggle',args);await frames();};
  await execute('effects.apply',{clips:[c.clip],effect:'gaussian_blur'});
  const fx=(await execute('sequence.inspect')).video[0].items[0].effects.find(e=>e.effect==='gaussian_blur').instanceId;
  await disclose({clip:c.clip,effectId:fx});
  await context(`${root}.effect.${fx}.header`);await menu(`animation.effect.${c.clip}.${fx}.menu.duplicate`);
  let current=(await execute('sequence.inspect')).video[0].items[0];
  const duplicates=current.effects.filter(e=>e.effect==='gaussian_blur');assert(duplicates.length===2&&duplicates[0].instanceId!==duplicates[1].instanceId,'Duplicate instance identity failed');
  const copy=duplicates.find(e=>e.instanceId!==fx).instanceId;
  await set(copy,'blurriness',27);assert(!eq((await effect(fx)).params.blurriness,(await effect(copy)).params.blurriness),'Duplicate edit crossed instances');
  // Both drawers independently disclose the duplicate's actual canonical parameter rows.
  await disclose({clip:c.clip,effectId:copy,param:'blurriness'});
  await execute('playhead.set',{frame:15});await execute('effects.addKeyframe',{clip:c.clip,effect:{instanceId:copy},param:'blurriness'});await frames();
  const copyTime=(await effect(copy)).params.blurriness.keyframes[0].time;
  await click(`${root}.effect.${copy}.param.blurriness.key.${copyTime}`);
  await capture('shared-duplicate-effects');
  await context(`${root}.effect.${copy}.header`);await menu(`animation.effect.${c.clip}.${copy}.menu.up`);
  current=(await execute('sequence.inspect')).video[0].items[0];assert(current.effects[0].instanceId===copy,'Reorder did not move the chosen instance');
  assert((await state())[0]?.effectId===copy&&(await state())[0]?.effect===0,'Reorder rebound the selected duplicate key');
  await context(`${root}.effect.${fx}.header`);await menu(`animation.effect.${c.clip}.${fx}.menu.remove`);
  assert(!(await execute('sequence.inspect')).video[0].items[0].effects.some(e=>e.instanceId===fx),'Remove targeted wrong duplicate');
  assert((await effect(copy)).params.blurriness.keyframes.length===1,'Remove damaged surviving duplicate');
  await execute('edit.undo');await frames();assert((await effect(fx)).instance_id===fx,'Undo did not restore persistent ID');
  await execute('edit.redo');await frames();assert((await effect(copy)).instance_id===copy,'Redo damaged surviving ID');
  checks.push('Physical instance menu duplicate/reorder/remove/undo/redo; independent data and disclosure');
  // Collapse added effects to keep the ordinary progressive layout, then work in Opacity.
  await disclose({clip:c.clip,effectId:copy});
  const opacity=current.effects.find(e=>e.effect==='opacity').instanceId;
  const canonical=async()=> (await effect(opacity)).params.opacity.keyframes;
  const prefix=`${root}.effect.${opacity}.param.opacity`;
  const keyId=t=>`${prefix}.key.${t}`;
  for(const [index,frame] of [15,45,75].entries()){
    await execute('playhead.set',{frame});await execute('effects.addKeyframe',{clip:c.clip,effect:{instanceId:opacity},param:'opacity'});await set(opacity,'opacity',40+index*20);
  }
  await disclose({clip:c.clip,effectId:opacity,param:'opacity'});
  let keys=await canonical();assert(keys.length===3,'Fixture did not create three real keys');
  for(let n=0;n<3;n++)await click(keyId(keys[n].time),n===1?8:n===2?2:0);
  assert((await state()).length===3,'Shift/Ctrl key multi-selection failed');
  let a=point(await reveal(keyId(keys[0].time)));const g=await geometry();
  // Measure the common ruler scale from two canonical keys, independent of monitor DPI/zoom.
  const b=point(await reveal(keyId(keys[1].time))),pixelsPerTick=(b.x-a.x)/(keys[1].time-keys[0].time);
  const delta=frameTicks*12; // Frame count comes from the actual canonical sequence rate.
  let end={x:a.x+delta*pixelsPerTick,y:a.y};
  const before=await canonical(),revision=(await geometry()).revision;await capture('shared-keyframes-before');
  await beginDrag(a,end);assert(eq(before,await canonical()),'Drag preview changed canonical keys');
  await capture('shared-group-drag-preview');await release(end);
  let moved=await canonical();report.sharedAnimation.group={before,after:moved,delta,revisionBefore:revision,revisionAfter:(await geometry()).revision};assert(moved.every((k,i)=>k.time===before[i].time+delta),'Group drag did not preserve exact spacing/frame ticks');
  const content=({time,...rest})=>rest;assert(moved.every((k,i)=>eq(content(k),content(before[i]))),'Group drag changed key values/interpolation/influence');
  assert((await geometry()).revision===revision+1,'Group drag was not one undoable edit');await capture('shared-keyframes-after');
  await execute('edit.undo');await frames();assert(eq(before,await canonical()),'Group undo did not restore exact keys');
  await execute('edit.redo');await frames();assert(eq(moved,await canonical()),'Group redo did not restore exact keys');
  checks.push('Individual/Shift/Ctrl selection, preview-only frame-snapped group drag, one-step exact undo/redo');
  // Cancel an actual second drag. Escape must leave both content and revision unchanged.
  await click(keyId(moved[0].time));a=point(await reveal(keyId(moved[0].time)));end={x:a.x+15,y:a.y};
  const cancelRevision=(await geometry()).revision;await beginDrag(a,end);await key('Escape','Escape',27);await release(end);
  assert(eq(moved,await canonical())&&(await geometry()).revision===cancelRevision,'Escape committed a key preview');checks.push('Physical Escape cancels drag without a project edit');
  // A single selected key must never overwrite an unselected destination key.
  await click(keyId(moved[0].time));a=point(await reveal(keyId(moved[0].time)));end=point(await reveal(keyId(moved[1].time)));
  await beginDrag(a,end);await release(end);assert(eq(moved,await canonical()),'Collision overwrote an unselected key');assert((await inspect()).ui.status.includes('occupies'),'Collision was not reported');checks.push('Occupied destination rejected atomically');
  // A real marquee starts in an empty key lane; ordinary clip selection is untouched.
  a=point(await reveal(keyId(moved[0].time)));const last=point(await reveal(keyId(moved[1].time)));
  const ma={x:a.x-15,y:a.y-7},mb={x:last.x+8,y:last.y+7};await beginDrag(ma,mb);await release(mb);
  assert((await state()).length===2,'Inline marquee did not select two keys');await capture('shared-marquee-selection');checks.push('Physical marquee selects canonical key references');
  const selected=await state();await call('ui.panel.show',{panel:'EffectControls'});await frames();assert(eq(selected,await state()),'Inspector erased inline selection');
  assert((await execute('state.inspect')).selected_effect.instanceId===opacity,'Inspector did not follow the actual effect instance');
  await capture('shared-inspector-selection');checks.push('Shared selection and selected effect survive inspector opening');
  // Filter changes are view state only and may not create undo records or keys.
  const filterRevision=(await geometry()).revision;
  for(const name of ['Animated','Modified','Selected','All'])await click(`${root}.filter.${name}`);
  assert((await geometry()).revision===filterRevision&&eq(moved,await canonical()),'Filter mutated canonical animation');checks.push('All/Animated/Modified/Selected filters preserve project and keys');
  await click(keyId(moved[0].time));
  const otherOpacity=other.effects.find(e=>e.effect==='opacity').instanceId;
  await execute('playhead.set',{time:other.start+127008000000});
  await execute('effects.addKeyframe',{clip:other.clip,effect:{instanceId:otherOpacity},param:'opacity'});
  await disclose({clip:other.clip,effectId:otherOpacity,param:'opacity'});
  const otherKeys=async()=> (await execute('effects.inspect',{clip:other.clip,effect:{instanceId:otherOpacity}})).params.opacity.keyframes;
  const otherBefore=await otherKeys();
  const otherId=`timeline.animation.clip.${other.clip}.effect.${otherOpacity}.param.opacity.key.${otherBefore[0].time}`;
  await click(otherId,2);assert((await state()).length===2,'Cross-track Ctrl selection failed');
  a=point(await reveal(otherId));end={x:a.x+pixelsPerTick*(frameTicks*6),y:a.y};
  const multiBefore=await canonical();await beginDrag(a,end);await release(end);
  const multiAfter=await canonical(),otherAfter=await otherKeys();
  assert(multiAfter[0].time===multiBefore[0].time+(frameTicks*6)&&otherAfter[0].time===otherBefore[0].time+(frameTicks*6),'Cross-track group lost source-time spacing');
  report.sharedAnimation.multitrack={before:multiBefore,after:multiAfter,otherBefore,otherAfter,selection:await state()};
  checks.push('Physical cross-track Ctrl selection and grouped drag use each clip source-time domain');
  const multi=await geometry();assert(multi.drawers.length===2,'Multiple tracks lost independent clip drawers');await capture('shared-multitrack');
  await send('Emulation.setDeviceMetricsOverride',{width:1280,height:800,deviceScaleFactor:1,mobile:false});await frames();await capture('shared-narrow');
  await send('Emulation.setDeviceMetricsOverride',{width:1656,height:894,deviceScaleFactor:1,mobile:false});await call('ui.set',{theme:'light'});await frames();await capture('shared-light');await call('ui.set',{theme:'dark'});
  await execute('file.openDemoProject');await frames();assert((await geometry()).drawers.length===0&&(await state()).length===0,'Document replacement reused old clip/key state');checks.push('Reused demo IDs clear old disclosure and selection');
  report.sharedAnimation.status='complete';
}
