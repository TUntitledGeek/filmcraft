import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';
import assert from 'node:assert/strict';
let Processor;const reports=[];
runInNewContext(readFileSync(new URL('../web/audio-worklet.js',import.meta.url),'utf8'),{
 AudioWorkletProcessor:class {constructor(){this.port={postMessage:m=>reports.push(m)}}},
 registerProcessor:(_,p)=>Processor=p,currentTime:0,
});
const audio=new Processor();
const block=()=>{const channels=[new Float32Array(128),new Float32Array(128)];audio.process([], [channels]);return channels};
audio.port.onmessage({data:{type:'reset',gen:1,active:true}});
block();assert.equal(audio.missing,0,'priming before the first data block is not output starvation');
const samples=Float32Array.from({length:512},(_,i)=>i%2?-.25:.5);
audio.port.onmessage({data:{type:'data',gen:1,samples}});
for(let i=0;i<2;i++){const [l,r]=block();assert(l.every(v=>v===.5));assert(r.every(v=>v===-.25))}
assert.equal(audio.played,256);
for(let i=0;i<4;i++){assert(block().every(c=>c.every(v=>v===0)))}
assert.equal(audio.played,256,'silence cannot advance the audio clock');
assert.equal(audio.missing,512);assert.equal(audio.starvations,4);
assert(reports.some(r=>r.missing>0),'real missing sample counts must reach the host');
audio.port.onmessage({data:{type:'reset',gen:2,active:false}});
audio.port.onmessage({data:{type:'data',gen:1,samples}});
for(let i=0;i<4;i++)block();
assert.equal(audio.played,0);assert.equal(audio.missing,0);assert.equal(audio.starvations,0);
console.log('Audio output: channel data, output-starvation counters, clock hold, generation rejection and idle/reset passed.');
