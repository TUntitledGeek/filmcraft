// Actual pointer movement over canvas hit zones while a foreground menu owns interaction.
export async function runTimelinePopupChecks({execute,call,send,capture,sleep,report}) {
  const assert=(ok,message)=>{if(!ok)throw Error(message);};
  const elements=()=>call('ui.elements');
  const element=async id=>(await elements()).find(e=>e.id===id);
  const point=e=>({x:e.rect[0]+e.rect[2]/2,y:e.rect[1]+e.rect[3]/2});
  const click=async(p,button='left')=>{
    await send('Input.dispatchMouseEvent',{type:'mouseMoved',...p,button:'none'});
    await call('ui.inspect');
    for(const type of ['mousePressed','mouseReleased']){await send('Input.dispatchMouseEvent',{type,...p,button,buttons:type==='mousePressed'?(button==='right'?2:1):0,clickCount:1});await call('ui.inspect');}
    await sleep(180);
  };
  const move=async p=>{await send('Input.dispatchMouseEvent',{type:'mouseMoved',...p,button:'none'});await sleep(180);};
  const menu='timeline.clipMenu.clip.speedDuration';
  const waitClosed=async()=>{for(let i=0;i<12;i++){await call('ui.inspect');if(!(await element(menu)))return true;await sleep(35);}return false;};
  const geometry=()=>call('ui.timeline.animation');
  report.popup={cases:[],complete:false};
  for(const expanded of [false,true]){
    await execute('file.openDemoProject');await call('ui.set',{workspace:'Clean Editing',mode:'edit'});
    const sequence=await execute('sequence.inspect'),clip=sequence.video[0].items[0];
    const root=`timeline.animation.clip.${clip.clip}`;
    if(expanded)await click(point(await element(root+'.toggle')));
    const body=await element(`timeline.clip.${clip.clip}`),before=(await geometry()).revision;
    await click(point(body),'right');assert(await element(menu),'Clip context menu did not open');
    const adjacent=await element(`timeline.animation.clip.${sequence.video[0].items[1].clip}.toggle`);
    const points=[{name:'clip edge',x:body.rect[0]+body.rect[2]-2,y:body.rect[1]+body.rect[3]/2},{name:'disclosure',...point(adjacent)}];
    if(expanded){const g=await geometry(),drawer=g.drawers.find(d=>d.clip===clip.clip).visible;points.push({name:'drawer',x:point(await element(menu)).x,y:drawer[1]+drawer[3]/2});}
    for(const p of points){await move({x:p.x,y:p.y});assert(await element(menu),'Menu disappeared over '+p.name);assert((await geometry()).revision===before,'Hovering a menu edited the project');}
    await capture(expanded?'popup-over-animation':'popup-over-clip-edge');
    await click(point(await element(menu)));let state=await call('ui.inspect');
    assert(state.ui.clip_dialog?.command==='clip.speedDuration','Settings command was intercepted by the canvas');
    assert((await geometry()).revision===before,'Opening Speed/Duration edited the project');
    const cancel=await element('speedDuration.cancel');assert(cancel,'Missing settings cancel');await click(point(cancel));
    await click(point(await element(`timeline.clip.${clip.clip}`)),'right');
    for(const type of ['keyDown','keyUp']){await send('Input.dispatchKeyEvent',{type,key:'Escape',code:'Escape',windowsVirtualKeyCode:27});await call('ui.inspect');}
    assert(await waitClosed(),'Escape did not close menu');
    await click(point(await element(`timeline.clip.${clip.clip}`)),'right');
    const outside=await geometry();
    await click({x:outside.content[0]+outside.content[2]-20,y:point(body).y});
    assert(await waitClosed(),'Clicking the canvas outside the menu did not close it');
    const second=await element(`timeline.clip.${sequence.video[0].items[1].clip}`);
    await click({x:second.rect[0]+second.rect[2]-2,y:second.rect[1]+second.rect[3]/2},'right');
    const trimId='timeline.editPointMenu.trimOut';assert(await element(trimId),'Right-clicking edge did not open edit-point settings');
    const g=await geometry();await move(expanded?{x:point(await element(trimId)).x,y:g.drawers[0].visible[1]+g.drawers[0].visible[3]/2}:point(adjacent));
    assert(await element(trimId),'Edit-point menu disappeared over disclosure/drawer');
    await click(point(await element(trimId)));
    const selected=(await execute('state.inspect')).edit_points;
    assert(selected.some(p=>p.clip===sequence.video[0].items[1].clip&&p.out&&p.kind==='trim'),'Edit-point command lost the original edge');
    report.popup.cases.push({expanded,hoverZones:points.map(p=>p.name),settings:true,escape:true,outside:true,editPointTarget:true,projectUnchanged:(await geometry()).revision===before});
    // Restore this fixture's view before starting the next independent project scenario.
    for(const view of (await geometry()).view.clips)await execute('timeline.animation.toggle',{clip:view.clip});
  }
  await execute('file.openDemoProject');await call('ui.set',{workspace:'Clean Editing',mode:'edit'});await execute('edit.deselectAll');
  report.popup.complete=true;
}
