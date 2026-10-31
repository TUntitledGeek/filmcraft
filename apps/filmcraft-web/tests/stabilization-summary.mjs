// Summarize completed matched runs without publishing private project contents,
// local paths, Chrome profile logs or raw recovery state.
import {readdirSync, readFileSync, writeFileSync, existsSync} from 'node:fs';
import {join} from 'node:path';

const root = process.argv[2];
const output = process.argv[3];
if (!root || !output) throw Error('Usage: node stabilization-summary.mjs <matched-directory> <output.json>');
const finite = values => values.filter(Number.isFinite).sort((a, b) => a - b);
const median = values => {
  const xs = finite(values), mid = Math.floor(xs.length / 2);
  return xs.length ? xs.length % 2 ? xs[mid] : (xs[mid - 1] + xs[mid]) / 2 : null;
};
const p95 = values => { const xs = finite(values); return xs.length ? xs[Math.ceil((xs.length - 1) * .95)] : null; };
const maximum = values => { const xs = finite(values); return xs.length ? xs.at(-1) : null; };
function audioSummary(samples) {
  const generations = new Map();
  for (const s of samples.filter(Boolean)) {
    if (!Number.isFinite(s.generation)) continue;
    const previous = generations.get(s.generation) ?? {generation: s.generation, missingFrames: 0, starvedQuanta: 0, playedFrames: 0};
    for (const key of ['missingFrames', 'starvedQuanta', 'playedFrames']) previous[key] = Math.max(previous[key], s[key] ?? 0);
    generations.set(s.generation, previous);
  }
  return generations.size ? [...generations.values()] : null;
}
const runs = [];
for (const name of readdirSync(root).sort()) {
  const match = /^(.*)-(before|after)-(\d+)$/.exec(name);
  const file = join(root, name, 'report.json');
  if (!match || !existsSync(file)) continue;
  const r = JSON.parse(readFileSync(file, 'utf8'));
  runs.push({
    case: match[1], build: match[2], repeat: Number(match[3]), status: r.status ?? 'failed',
    at: r.at, wasmSha256: r.wasmSha256, projectSha256: r.projectSha256,
    browser: r.browser, hardware: r.hardware, adapter: r.adapter, viewport: r.viewport,
    sequence: r.sequence, effectIdentity: r.effectIdentity, initialPictureMs: r.initialPictureMs,
    frameWorkers: r.info?.frameWorkers, crossOriginIsolated: r.info?.crossOriginIsolated,
    playback: (r.playback ?? []).map(p => ({
      phase: p.label, elapsedWallMs: p.elapsedWallMs,
      presentationChanges: p.after.playback.presented,
      presentationChangesPerSecond: p.after.playback.presented * 1000 / p.elapsedWallMs,
      shown: p.after.playback.shown, dropped: p.after.playback.dropped,
      lagFramesP95: p95(p.browser.samples.map(s => s.monitor.playheadFrame - s.monitor.frame)),
      rafP95Ms: p.browser.p95Ms, rafMaxMs: p.browser.maxMs,
      longestPlaybackTaskMs: maximum([0, ...p.browser.longTasks.map(t => t.duration)]),
      playbackTasksOver100Ms: p.browser.longTasks.filter(t => t.duration > 100).length,
      pauseAckMs: p.pauseMs, audioOutput: audioSummary([...p.browser.samples.map(s => s.audio), p.info?.audioOutput]),
      mainWasmCapacityBytes: p.info?.wasmMemoryBytes,
      workerWasmCapacityBytes: p.info?.frameWorker?.media?.wasmMemoryBytes,
      frames: p.after.frames, worker: p.info?.frameWorker,
    })),
    selectionUiUpperBoundMs: r.interactions?.filter(i => i.kind === 'pointerSelection').map(i => i.ms),
    selectionSeekCacheState: 'Not asserted; requested range overlaps playback range.',
    undoTrim: r.interactions?.find(i => i.kind === 'undoTrim'),
    parameterEdits: r.parameterEdits, lumetriDrag: r.lumetriDrag, checks: r.checks,
    browserExceptions: r.errors.length,
  });
}
const grouped = new Map();
for (const r of runs) {
  const key = `${r.case}|${r.build}`;
  if (!grouped.has(key)) grouped.set(key, []);
  grouped.get(key).push(r);
}
const summaries = [...grouped].map(([key, attempts]) => {
  const rs = attempts.filter(r => r.status === 'passed');
  const phases = rs.flatMap(r => r.playback);
  return {
    key, attempts: attempts.length, passed: rs.length,
    coldChangesPerSecond: median(phases.filter(p => p.phase === 'cold').map(p => p.presentationChangesPerSecond)),
    warmChangesPerSecond: median(phases.filter(p => p.phase === 'warm').map(p => p.presentationChangesPerSecond)),
    rafP95MedianMs: median(phases.map(p => p.rafP95Ms)),
    selectionUiUpperBoundP95Ms: p95(rs.flatMap(r => r.selectionUiUpperBoundMs ?? [])),
    parameterUiUpperBoundP95Ms: p95(rs.flatMap(r => r.parameterEdits?.map(p => p.visibleUiUpperBoundMs) ?? [])),
    parameterPictureP95Ms: p95(rs.flatMap(r => r.parameterEdits?.map(p => p.pictureMs) ?? [])),
    pauseAckP95Ms: p95(phases.map(p => p.pauseAckMs)),
    longestPlaybackTaskMs: maximum(phases.map(p => p.longestPlaybackTaskMs)),
    playbackTasksOver100Ms: phases.reduce((n, p) => n + p.playbackTasksOver100Ms, 0),
    mainWasmCapacityMaxBytes: maximum(phases.map(p => p.mainWasmCapacityBytes)),
    workerWasmCapacityMaxBytes: maximum(phases.map(p => p.workerWasmCapacityBytes)),
    outputMissingFrames: phases.some(p => p.audioOutput) ? phases.reduce((n, p) => n + (p.audioOutput ?? []).reduce((a, g) => a + g.missingFrames, 0), 0) : null,
  };
});
writeFileSync(output, JSON.stringify({
  schema: 1, sourceCheckpoint: '563e36b5',
  method: 'Three interleaved fresh-profile repeats; first/second playback pass. See performance-stabilization-reproduction.md.',
  limitations: [
    'Presentation changes are not globally unique-frame FPS or rAF.',
    'UI selection times include automation and two rAFs; strict 50 ms feedback is not certified.',
    'Selection seeks overlap playback and do not assert cache misses.',
    'Long Tasks cover playback, not the entire editing phase.',
    'WASM capacity is not resident process memory; missing baseline memory/audio telemetry is null.',
  ], runs, summaries,
}, null, 2) + '\n');
console.log(`${runs.length} reports, ${runs.filter(r => r.status === 'passed').length} passed; wrote ${output}`);
