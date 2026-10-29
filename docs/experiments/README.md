# Deferred optional WASM threading experiment

Snapshot base: 1a2ab681c1ec9d9c5624500da826e9d89b6941a7. The patch reconstructs the complete experimental working tree at that base, including the decoder-policy and research files that were also under development. Apply only to a separate checkout of that exact base; it is not a patch to apply blindly to the final performance branch.

The optional atomics artifact compiled successfully with nightly-2026-10-28 and wasm-bindgen-rayon 1.3.0. It targets ordinary Chrome, requires site isolation headers, and includes a standard fallback. Compilation is not runtime verification. The experiment was withdrawn from active source before final preservation because browser Blob/WebCodecs state is event-loop-local, while audio strip and multicamera source traversal can execute in Rayon workers. That boundary needs an explicit source-I/O design and audible/output regression tests before enabling a shared worker pool. No unsafe Send/Sync workaround is acceptable.

The experiment also isolates native blocking GPU export setup and browser egui repaint callbacks. It does not implement a dedicated render worker, source-state message bridge, verified threaded audio or a proven frame-pacing improvement. No threaded performance claim is made.

Approval of the optional runtime and pinned toolchain was received; the separate installed toolchain does not change the standard compiler or require a custom browser. Final active source builds the standard artifact and does not depend on this runtime.
