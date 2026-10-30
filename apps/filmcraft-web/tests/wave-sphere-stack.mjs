// Functional acceptance in an isolated Chrome session. All edits use canonical engine
// commands and persistent effect IDs; no recovery snapshot or private project is needed.
export async function runWaveSphereStackChecks({call,execute,capture,sleep,report}) {
  const assert=(value,message)=>{if(!value)throw new Error(message);};
  await execute('file.openDemoProject');
  await call('ui.set',{mode:'edit',workspace:'Clean Editing',program:{res:'Half',paused_res:'Half',zoom:.5}});
  const sequence=()=>execute('sequence.inspect');
  const clip=(await sequence()).video.flatMap(t=>t.items).find(c=>c.clip)?.clip;
  assert(clip,'No demo picture clip');
  const inspect=async()=>(await sequence()).video.flatMap(t=>t.items).find(c=>c.clip===clip);
  const ids={};
  for(const [effect,params] of [
    ['lumetri',{temperature:73.7}],
    ['gaussian_blur',{blurriness:17.75}],
    ['wave_warp',{height:10,width:40,direction:90,speed:1}],
    ['spherize',{radius:0}],
  ]){
    let instance=(await inspect()).effects.find(e=>e.effect===effect);
    if(!instance){await execute('effects.apply',{clips:[clip],effect});instance=(await inspect()).effects.find(e=>e.effect===effect);}
    assert(instance?.instanceId,'Missing stable '+effect+' identity');
    ids[effect]=instance.instanceId;
    for(const [param,value] of Object.entries(params))await execute('effects.setParam',{clip,effect:{instanceId:instance.instanceId},param,value});
  }
  const wait=async(frame)=>{
    const started=performance.now();let stats;
    while(performance.now()-started<15000){
      stats=await call('perf.stats');
      if(stats.monitor.frame===frame&&stats.monitor.revision===stats.monitor.projectRevision&&stats.monitor.renderScale===.5)return stats;
      await sleep(15);
    }
    throw new Error('Enabled stack did not produce the requested picture: '+JSON.stringify(stats.monitor));
  };
  const point=await call('ui.timeline.locate',{clip});await call('ui.click',point);
  await execute('playhead.set',{frame:75});await wait(75);
  const baseline=JSON.stringify((await inspect()).effects),edits=[];
  const gpuBefore=await call('perf.stats');
  await execute('playhead.set',{frame:76});
  const gpuAfter=await wait(76);
  assert(gpuAfter.gpu.effectsStatus==='ready'&&gpuAfter.gpu.gpuEffectLayers>gpuBefore.gpu.gpuEffectLayers,'The complete enabled chain fell back from GPU effects');
  for(const [effect,param,value] of [['lumetri','temperature',75.7],['gaussian_blur','blurriness',18.75],['wave_warp','height',13],['spherize','radius',160]]){
    const started=performance.now();
    await execute('effects.setParam',{clip,effect:{instanceId:ids[effect]},param,value});
    const rendered=await wait(76);
    edits.push({effect,instanceId:ids[effect],param,value,acknowledgedPictureMs:performance.now()-started,monitor:rendered.monitor});
    await execute('edit.undo');await wait(76);
    assert(JSON.stringify((await inspect()).effects)===baseline,'Undo changed effect identity, order or parameters');
  }
  await execute('edit.redo');await wait(76);
  const nonzeroSphere=await execute('effects.inspect',{clip,effect:{instanceId:ids.spherize}});
  await execute('edit.undo');await wait(76);
  assert(JSON.stringify((await inspect()).effects)===baseline,'Redo/undo did not restore the complete effect stack');
  assert((await inspect()).effects.filter(e=>Object.values(ids).includes(e.instanceId)).every(e=>e.enabled),'An effect was bypassed');
  report.waveSphereStack={status:'passed',clip,ids,gpuLayerDelta:gpuAfter.gpu.gpuEffectLayers-gpuBefore.gpu.gpuEffectLayers,edits,nonzeroSphere,undoRedoPreserved:true,
    note:'Functional acceptance timings include control/polling overhead; use the controlled benchmark for performance claims.'};
  await call('ui.panel.show',{panel:'EffectControls'});
  await capture('wave-sphere-enabled-stack');
}
