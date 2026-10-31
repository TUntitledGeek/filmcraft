// Each worker owns an independent, ordinary WASM instance. The compiled module
// is cloned, not its memory; no atomics, SharedArrayBuffer or custom browser.
let api;
let busy = false;
self.onmessage = async ({ data }) => {
  try {
    if (data.type === "init") {
      if (api) throw new Error("Frame worker was already initialized");
      api = await import(data.glue);
      await api.default({ module_or_path: data.module });
      await api.frame_worker_initialize(data.files, data.search);
      self.postMessage({ type: "ready" });
    } else if (data.type === "frame") {
      if (!api || busy) throw new Error("Frame worker protocol violation");
      busy = true;
      api.frame_worker_begin(data.task, data.project, data.previews, data.retained, data.capabilities);
      // I/O and WebCodecs callbacks must get turns between pending frame attempts.
      let result;
      const deadline = performance.now() + 120000;
      while (!(result = api.frame_worker_poll())) {
        if (performance.now() > deadline) throw new Error("Frame worker timed out waiting for media");
        await new Promise(resolve => setTimeout(resolve, 8));
      }
      self.postMessage({ type: "frame", id: data.id, ...result }, result.buffers.map(b => b.buffer));
      busy = false;
    }
  } catch (e) {
    busy = false;
    self.postMessage({ type: "error", id: data.id, error: String(e?.stack || e) });
  }
};
