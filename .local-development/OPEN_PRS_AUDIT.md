# FilmCraft Open Pull Requests Audit (Windows & Reliability Candidates)

Analyzed 559 open pull requests on `storytold/filmcraft` as of October 2026.

## Category: WINDOWS (18 PRs)

| PR | Title | Author | Branch | Matched Keywords |
| :--- | :--- | :--- | :--- | :--- |
| [#2492](https://github.com/storytold/filmcraft/pull/2492) | Docs: update Media Foundation encoder and GPU export notes | `jahvari` | `codex/mf-encoder-docs` | media foundation |
| [#2488](https://github.com/storytold/filmcraft/pull/2488) | Windows: add Media Foundation H.264 and HEVC export | `jahvari` | `codex/mf-encoder-core` | windows, media foundation |
| [#2487](https://github.com/storytold/filmcraft/pull/2487) | AV1: integrate guarded decoding with Media Foundation | `jahvari` | `codex/av1-mf-integration` | media foundation |
| [#2431](https://github.com/storytold/filmcraft/pull/2431) | platform: NVDEC on Windows for 4:2:2 and 10-bit H.264 camera footage (draft; builds on the h264 PR) | `kottav` | `nvdec-windows-high-bit-depth` | windows |
| [#2417](https://github.com/storytold/filmcraft/pull/2417) | audio: work around distorted Razer 7.1 WASAPI output | `trikesh32` | `fix/2411-razer-surround-audio` | wasapi |
| [#2169](https://github.com/storytold/filmcraft/pull/2169) | Fix #1994: persist native window geometry on Windows | `TheGalitube` | `fix-issue-1994-window-geometry` | windows |
| [#2143](https://github.com/storytold/filmcraft/pull/2143) | feat(installer): unify Windows setup and completion screens | `skylartaylor` | `codex/windows-installer-latest-20261014-01` | windows |
| [#2086](https://github.com/storytold/filmcraft/pull/2086) | WIN-07: avoid Windows rustfmt command-length failures | `jahvari` | `fix/windows-rustfmt-batching-win07` | windows |
| [#1954](https://github.com/storytold/filmcraft/pull/1954) | Decode Blackmagic RAW (.braw) through Blackmagic's runtime (Linux, Windows, macOS) | `txiki739` | `feat/braw-sdk` | windows |
| [#1811](https://github.com/storytold/filmcraft/pull/1811) | Windows: guard initial AV1 color metadata for hardware decoding | `jahvari` | `fix/av1-initial-metadata-1678` | windows |
| [#1575](https://github.com/storytold/filmcraft/pull/1575) | Media Foundation hardware H.264 / HEVC encoding on Windows (#30) | `ComfyChloe` | `windows/media-foundation-encoder` | windows, media foundation |
| [#1170](https://github.com/storytold/filmcraft/pull/1170) | Prefer Microsoft YaHei UI for Windows Simplified Chinese | `sch401` | `fix/windows-chinese-font-and-timeline-scrubbing` | windows |
| [#1128](https://github.com/storytold/filmcraft/pull/1128) | feat(platform): Intel Quick Sync Video (QSV) hardware encoding for Linux and Windows | `AnarchByte0` | `feat/intel-qsv-encoder` | windows |
| [#1021](https://github.com/storytold/filmcraft/pull/1021) | perf(windows): share decoded frames with the DX12 compositor | `WavJaby` | `perf/windows-resident-split` | windows |
| [#966](https://github.com/storytold/filmcraft/pull/966) | CI: FreeBSD and Windows ARM64 nightly and on release, not per PR | `echelon` | `ci-speedup-nightly-platform-checks` | windows |
| [#880](https://github.com/storytold/filmcraft/pull/880) | feat(platform): AMD AMF hardware encoding for Linux and Windows | `AnarchByte0` | `feat/amf-and-av1-encoder` | windows |
| [#583](https://github.com/storytold/filmcraft/pull/583) | Windows source-checkout launcher with --log mode | `didpublishing` | `windows-launcher` | windows |
| [#152](https://github.com/storytold/filmcraft/pull/152) | Windows: zero-copy hardware decode (D3D11 surface to wgpu DX12) | `SaymonMaster14` | `feat/windows-zero-copy-decode` | windows, d3d11 |

## Category: GPU_PERF (42 PRs)

| PR | Title | Author | Branch | Matched Keywords |
| :--- | :--- | :--- | :--- | :--- |
| [#2492](https://github.com/storytold/filmcraft/pull/2492) | Docs: update Media Foundation encoder and GPU export notes | `jahvari` | `codex/mf-encoder-docs` | gpu |
| [#2491](https://github.com/storytold/filmcraft/pull/2491) | UI: report GPU compositor startup failures | `jahvari` | `codex/ui-compositor-status` | gpu |
| [#2490](https://github.com/storytold/filmcraft/pull/2490) | GPU: bound export read-back waits | `jahvari` | `codex/gpu-bounded-readback` | gpu, readback, read-back |
| [#2418](https://github.com/storytold/filmcraft/pull/2418) | #1616: perf.stats says how many export frames the GPU rendered and why it is off when it is | `00xJS` | `fix/1616-gpu-render-stats` | gpu, perf |
| [#2407](https://github.com/storytold/filmcraft/pull/2407) | Playback: Linux audio over the PulseAudio native protocol, with reconnects (#106, #2117) | `adventurelands` | `omarchy-audio` | playback |
| [#2366](https://github.com/storytold/filmcraft/pull/2366) | perf(ui): Trim Monitor pictures at a cut inside one take stop cancelling each other | `inspiretelapps` | `perf/frame-request-views` | perf |
| [#2293](https://github.com/storytold/filmcraft/pull/2293) | ui: show active state for loop playback | `VyxorDev` | `fix/2286-loop-active-state` | playback |
| [#2233](https://github.com/storytold/filmcraft/pull/2233) | perf(export): add SDR RGBA8 GPU fast path | `ZpdyGonzales` | `feature/sdr-rgba8-export` | gpu, perf, fast path |
| [#2193](https://github.com/storytold/filmcraft/pull/2193) | Overlap VA-API readback with decoding on Linux | `alexandrucaraus` | `feat/linux-video-preview` | readback |
| [#2181](https://github.com/storytold/filmcraft/pull/2181) | fix(render): reduce CPU load during transition playback | `TheGalitube` | `codex/issue-2098-cpu-playback` | playback |
| [#2139](https://github.com/storytold/filmcraft/pull/2139) | fix(gpu): Intel Gen 9 to Gen 11 integrated graphics draw through DirectX 12 (#512, #1425) | `derektrimm` | `fix/512-intel-integrated-dx12` | gpu |
| [#2138](https://github.com/storytold/filmcraft/pull/2138) | fix(gpu): FXC compiles the effect stage, so DX12 keeps GPU effects (#1616, #1425, #1458) | `derektrimm` | `fix/fxc-unsharp-vector-write` | gpu |
| [#2133](https://github.com/storytold/filmcraft/pull/2133) | M5 (#666) updated for main: GPU Lumetri LUTs on Apple Silicon, GPU by default for unpreset exports | `earlvanze` | `perf/gpu-lut-666-on-main` | gpu, perf |
| [#2119](https://github.com/storytold/filmcraft/pull/2119) | export: GPU-rendered SDR exports read back the finished sRGB RGBA8 frame (1.6× faster NVENC export) | `MasakatsuFunaki` | `export-gpu-rgba8-readback` | gpu, readback |
| [#2111](https://github.com/storytold/filmcraft/pull/2111) | Fix potential GPU export deadlock by preparing frame plans outside renderer pool | `ZpdyGonzales` | `fix/gpu-export-deadlock-upstream` | gpu |
| [#2025](https://github.com/storytold/filmcraft/pull/2025) | GPU export fast path and mask support | `TheGalitube` | `main` | gpu, fast path |
| [#1978](https://github.com/storytold/filmcraft/pull/1978) | Restore mixer throughput without changing DSP output | `skylartaylor` | `codex/independent-mixer-throughput-20261015` | throughput |
| [#1973](https://github.com/storytold/filmcraft/pull/1973) | Pause playback on Ripple Trim Previous Edit to Playhead | `Hezzym` | `main` | playback |
| [#1967](https://github.com/storytold/filmcraft/pull/1967) | ui: playhead.set while playing moves the playhead and playback continues from it (#1622) | `midasmatterhorn` | `fix/issue-1622-playhead-set-during-playback` | playback |
| [#1957](https://github.com/storytold/filmcraft/pull/1957) | render: Transform with its own shutter angle stays on the CPU (the GPU path dropped its motion blur) | `schnierk` | `fix/transform-shutter-angle-on-gpu` | gpu |
| [#1866](https://github.com/storytold/filmcraft/pull/1866) | perf: Warp Stabilizer works out the corrections of a clip once per set of settings | `Guilhermeb-Ferrarezi` | `perf/stabilizer-cache-corrections` | perf |
| [#1814](https://github.com/storytold/filmcraft/pull/1814) | codecs: eliminate gap/dropout clicks in Matroska PCM audio playback | `woldercraft` | `fix/matroska-pcm-audio-continuity` | playback |
| [#1784](https://github.com/storytold/filmcraft/pull/1784) | perf: auto-scale memory budgets, GOP cache, and GPU upload cache to host RAM/VRAM | `ProgrammerDSJ` | `perf/auto-scale-memory-and-cache` | gpu, perf |
| [#1753](https://github.com/storytold/filmcraft/pull/1753) | Pack opaque SDR export readback on the GPU | `Gustitolito` | `codex/export-pipeline-throughput` | gpu, readback, throughput |
| [#1744](https://github.com/storytold/filmcraft/pull/1744) | gpu tests: one GPU test at a time in the lib test binary | `iannitram` | `upstream-pr/gpu-tests-serialize` | gpu |
| [#1719](https://github.com/storytold/filmcraft/pull/1719) | Program: add preview playback speed selector | `andreivincit` | `contribution/review-speed` | playback |
| [#1661](https://github.com/storytold/filmcraft/pull/1661) | playback: Stop can return the playhead to where playback started, and Ctrl+Shift+Space stops the other way | `ReReddo` | `feat/stop-returns-to-start` | playback |
| [#1649](https://github.com/storytold/filmcraft/pull/1649) | scrub bars: dragging the playhead keeps the audio stream open instead of stopping it every move | `ReReddo` | `fix/scrub-keeps-audio-stream` | scrub |
| [#1605](https://github.com/storytold/filmcraft/pull/1605) | timecodes: Effect Controls, the Audio Track Mixer and the Reference Monitor scrub and take a typed time; one font for them all | `ReReddo` | `fix/effect-controls-timecode-scrub` | scrub |
| [#1560](https://github.com/storytold/filmcraft/pull/1560) | gpu: interpolate a scaled-up Y'CbCr layer in linear light, as the CPU does (#1558) | `marcoarnulfo` | `fix/untagged-portrait-gpu-matrix` | gpu |
| [#1559](https://github.com/storytold/filmcraft/pull/1559) | export: GPU compositing in the standalone CLI, GPU status in export results, renderer-pool hang (#799) | `bsoulai` | `bsoulai/headless-gpu-799` | gpu |
| [#1446](https://github.com/storytold/filmcraft/pull/1446) | fix(monitor): the playback and paused resolution set the rendered frame size | `marcoarnulfo` | `fix/preview-resolution` | playback |
| [#1363](https://github.com/storytold/filmcraft/pull/1363) | perf(av1): optimize decoder allocations, zero-fill elimination and DC transforms | `AnarchByte0` | `perf/av1-decoder-optimizations` | perf |
| [#1284](https://github.com/storytold/filmcraft/pull/1284) | GPU export: 4:2:0 frames straight from the GPU (colour conversion on the GPU) (#30) | `LloydNicholson` | `gpu-yuv-export` | gpu |
| [#1114](https://github.com/storytold/filmcraft/pull/1114) | feat(web): share resident GPU sources and lazy CPU fallback | `WavJaby` | `perf/web-resident-frames` | gpu, perf |
| [#1076](https://github.com/storytold/filmcraft/pull/1076) | #1039: HEVC decodes pictures on several threads inside (WPP rows, tiles, filters next to parsing) | `vangel76` | `perf-hevc-wpp` | perf |
| [#1021](https://github.com/storytold/filmcraft/pull/1021) | perf(windows): share decoded frames with the DX12 compositor | `WavJaby` | `perf/windows-resident-split` | perf |
| [#883](https://github.com/storytold/filmcraft/pull/883) | h264: safe-SIMD 6-tap luma spike (decoder throughput half of #848) | `vbasky` | `feat/848-h264-safe-simd` | throughput |
| [#666](https://github.com/storytold/filmcraft/pull/666) | M5: Accelerate Apple Silicon media and GPU effect pipelines | `notnotshuts` | `perf/apple-silicon-metal-pipeline` | gpu, perf |
| [#332](https://github.com/storytold/filmcraft/pull/332) | Playback: Linux audio over the PulseAudio native protocol, pure Rust (#106) | `Chace-Berry` | `linux-pipewire-audio` | playback |
| [#152](https://github.com/storytold/filmcraft/pull/152) | Windows: zero-copy hardware decode (D3D11 surface to wgpu DX12) | `SaymonMaster14` | `feat/windows-zero-copy-decode` | wgpu, zero-copy |
| [#37](https://github.com/storytold/filmcraft/pull/37) | FreeBSD CI + cross-platform fixes it found (shortcuts, GPU effect fallback) | `echelon` | `pr/freebsd` | gpu |

## Category: CODECS_MEDIA (77 PRs)

| PR | Title | Author | Branch | Matched Keywords |
| :--- | :--- | :--- | :--- | :--- |
| [#2492](https://github.com/storytold/filmcraft/pull/2492) | Docs: update Media Foundation encoder and GPU export notes | `jahvari` | `codex/mf-encoder-docs` | encoder |
| [#2488](https://github.com/storytold/filmcraft/pull/2488) | Windows: add Media Foundation H.264 and HEVC export | `jahvari` | `codex/mf-encoder-core` | encoder, hevc, h\.?264 |
| [#2487](https://github.com/storytold/filmcraft/pull/2487) | AV1: integrate guarded decoding with Media Foundation | `jahvari` | `codex/av1-mf-integration` | av1 |
| [#2486](https://github.com/storytold/filmcraft/pull/2486) | AV1: guard native metadata adoption and replay | `jahvari` | `codex/av1-replay-policy` | av1 |
| [#2485](https://github.com/storytold/filmcraft/pull/2485) | AV1: add bounded sequence-header inspection | `jahvari` | `codex/av1-header-inspection` | av1 |
| [#2475](https://github.com/storytold/filmcraft/pull/2475) | #2469: the media pool closes least recently used decoded stills past a budget | `Felix-Helleckes` | `fix/2469-media-pool-budget` | media pool |
| [#2451](https://github.com/storytold/filmcraft/pull/2451) | Timeline: colour labels for tracks (#1186) | `Felix-Helleckes` | `feat/1186-track-color-labels` | color |
| [#2431](https://github.com/storytold/filmcraft/pull/2431) | platform: NVDEC on Windows for 4:2:2 and 10-bit H.264 camera footage (draft; builds on the h264 PR) | `kottav` | `nvdec-windows-high-bit-depth` | h\.?264 |
| [#2415](https://github.com/storytold/filmcraft/pull/2415) | #2340: uncompressed QuickTime video (raw 24-bit RGB, 2vuy, v210) decodes | `Felix-Helleckes` | `fix/2340-uncompressed-mov` | mov |
| [#2403](https://github.com/storytold/filmcraft/pull/2403) | #2195: relative media URLs in imported timelines are percent-decoded | `Felix-Helleckes` | `fix/2195-decode-relative-media-urls` | decode |
| [#2393](https://github.com/storytold/filmcraft/pull/2393) | #1123: Effect Controls' LUT rows pick a LUT | `Felix-Helleckes` | `fix/1123-effect-controls-lut-menus` | lut |
| [#2391](https://github.com/storytold/filmcraft/pull/2391) | #1422: Matroska audio without a decoder is labelled unsupported | `Felix-Helleckes` | `fix/1422-mkv-unsupported-audio-label` | decoder |
| [#2378](https://github.com/storytold/filmcraft/pull/2378) | Lumetri: Auto Color Correction for clips or the whole sequence (#1152) | `Felix-Helleckes` | `feat/1152-auto-basic-correction` | color |
| [#2372](https://github.com/storytold/filmcraft/pull/2372) | fix(hevc tests): skip Main10 fixtures an 8-bit-only libx265 cannot encode | `adventurelands` | `hevc-x265-8bit` | encode, hevc |
| [#2359](https://github.com/storytold/filmcraft/pull/2359) | Add a hex field to colour parameters in Effect Controls | `Felix-Helleckes` | `feat/1561-hex-input-for-color-params` | color |
| [#2303](https://github.com/storytold/filmcraft/pull/2303) | AVI: decode H.264 and HEVC with their parameter sets in the stream header | `tylerjenningsw` | `fix/avi-h264-hevc-stream-setup` | decode, hevc, h\.?264 |
| [#2301](https://github.com/storytold/filmcraft/pull/2301) | h264: High 10 / High 4:2:2 decoding, intra and inter (Sony XAVC S-I and S) | `kottav` | `h264-high-bit-depth-nvdec` | h\.?264 |
| [#2239](https://github.com/storytold/filmcraft/pull/2239) | Fix phone MP4 timestamp frame rates | `TheGalitube` | `codex/fix-issue-1700-phone-timestamp-rate` | mp4, timestamp, phone |
| [#2221](https://github.com/storytold/filmcraft/pull/2221) | Avoid unnecessary hardware->software fallback for AV1 when only colour description differs (corrected AV1 spec) | `pixelsdontmove` | `fix/av1-color-description` | av1, color |
| [#2189](https://github.com/storytold/filmcraft/pull/2189) | fix(export): unpreset H.264 exports start from the default preset | `derektrimm` | `fix/export-default-preset` | h\.?264 |
| [#2180](https://github.com/storytold/filmcraft/pull/2180) | fix(platform): decode VP9 in hardware through VideoToolbox on macOS | `Andrey-Ultra` | `vp9-videotoolbox` | decode, vp9 |
| [#2170](https://github.com/storytold/filmcraft/pull/2170) | Fix #1996: reflect selected text fill in color picker | `TheGalitube` | `fix-issue-1996-selected-text-color-picker` | color |
| [#2137](https://github.com/storytold/filmcraft/pull/2137) | fix(macos): decode interlaced H.264 with VideoToolbox's software decoder (#1750) | `ssandweiss` | `fix/1750-videotoolbox-interlaced` | decode, decoder, h\.?264 |
| [#2135](https://github.com/storytold/filmcraft/pull/2135) | #1984: silence concealing an MP4 audio packet that will not decode is bounded (its duration is the file's) | `00xJS` | `fix/1984-aac-fallback-allocation` | decode, aac, mp4 |
| [#2133](https://github.com/storytold/filmcraft/pull/2133) | M5 (#666) updated for main: GPU Lumetri LUTs on Apple Silicon, GPU by default for unpreset exports | `earlvanze` | `perf/gpu-lut-666-on-main` | lut |
| [#2113](https://github.com/storytold/filmcraft/pull/2113) | Write the output bit depth in exported .3dl files | `Silvertree2010` | `fix/h1773-a-dark-lut-exported-to-3dl-is-re` | lut |
| [#2092](https://github.com/storytold/filmcraft/pull/2092) | Add native NTSC DV25 decoding for MOV imports | `andymenard` | `codex/ntsc-dv25-pr` | mov |
| [#2076](https://github.com/storytold/filmcraft/pull/2076) | HEVC: select the x86-v3 inverse transform at runtime, retaining the baseline path | `fanthomas-creator` | `codex/hevc-runtime-cpu-review` | hevc |
| [#2075](https://github.com/storytold/filmcraft/pull/2075) | fix(macOS): show file dialogs as sheets instead of a nested modal run loop (#2018) | `vark-debug` | `fix/lut-dialog-reentrancy` | lut |
| [#2053](https://github.com/storytold/filmcraft/pull/2053) | feat(timeline): add wheel zoom and waveform color preferences | `channelramble` | `feat/timeline-wheel-and-waveform-colors` | color |
| [#1987](https://github.com/storytold/filmcraft/pull/1987) | codecs: avoid duration-sized silence allocation for damaged MP4 audio | `nuclearspike` | `codex/security-mp4-audio-duration` | mp4 |
| [#1954](https://github.com/storytold/filmcraft/pull/1954) | Decode Blackmagic RAW (.braw) through Blackmagic's runtime (Linux, Windows, macOS) | `txiki739` | `feat/braw-sdk` | decode |
| [#1952](https://github.com/storytold/filmcraft/pull/1952) | codecs: HEVC 4:2:2 8/10-bit decoding (Main 4:2:2 10 / RExt profile), bit-exact (#626, G3) | `melchiaaronzedek` | `feat/hevc-422-10bit` | hevc |
| [#1928](https://github.com/storytold/filmcraft/pull/1928) | #1216: the decoders and the H.264 encoder an export starts keep to its maxThreads | `Guilhermeb-Ferrarezi` | `feat/export-thread-cap-codecs-1216` | encoder, h\.?264 |
| [#1880](https://github.com/storytold/filmcraft/pull/1880) | Rename 'Lumetri' UI terms to 'Color Grade' and 'Video Scopes' | `alzithetrivialmind` | `fix/1868-rename-lumetri-trademark` | color |
| [#1858](https://github.com/storytold/filmcraft/pull/1858) | HEVC Main software encoder (I/P, single-slice) with export wiring | `lu123bing` | `feat/hevc-software-encoder` | encoder, hevc |
| [#1835](https://github.com/storytold/filmcraft/pull/1835) | fix(codecs): read AC-3/E-AC-3 MP4 layout from dac3/dec3 (#1701) | `earlvanze` | `fix/1701-mp4-ac3-layout-upstream` | mp4 |
| [#1829](https://github.com/storytold/filmcraft/pull/1829) | fix(codecs): read the frame rate phone clips average to (#1700) | `earlvanze` | `fix/1700-ms-frame-times` | phone |
| [#1811](https://github.com/storytold/filmcraft/pull/1811) | Windows: guard initial AV1 color metadata for hardware decoding | `jahvari` | `fix/av1-initial-metadata-1678` | av1, color |
| [#1795](https://github.com/storytold/filmcraft/pull/1795) | Hostile-input hardening for video decoders (AV1 panic + oversize allocation, MPEG-2 allocation) and fuzz tests | `g1obalworlds` | `fix/hostile-input-video-codecs` | av1 |
| [#1778](https://github.com/storytold/filmcraft/pull/1778) | Ultra Key: colour-difference matte for green/blue screens; red edges solved from their surroundings | `Guilhermeb-Ferrarezi` | `fix/ultra-key-color-difference` | color |
| [#1774](https://github.com/storytold/filmcraft/pull/1774) | HW6: VA-API hardware H.264/HEVC export encoding on Linux | `earlvanze` | `pr408-rebased-from-main` | hevc, h\.?264 |
| [#1764](https://github.com/storytold/filmcraft/pull/1764) | codecs: detect explicit Apple Log metadata in MOV | `BitL8-ByteShort` | `jorvek/filmcraft-987-apple-log` | mov |
| [#1751](https://github.com/storytold/filmcraft/pull/1751) | codecs: preserve MP4 Dolby surround channels | `BitL8-ByteShort` | `jorvek/filmcraft-1701-surround` | mp4 |
| [#1745](https://github.com/storytold/filmcraft/pull/1745) | platform: VA-API H.264 stays in hardware after an open-GOP seek on Mesa radeonsi | `iannitram` | `upstream-pr/vaapi-open-gop` | h\.?264 |
| [#1742](https://github.com/storytold/filmcraft/pull/1742) | codecs: report AV1 and Matroska pixel formats | `BitL8-ByteShort` | `jorvek/filmcraft-1702-pixel-format` | av1 |
| [#1739](https://github.com/storytold/filmcraft/pull/1739) | fix(codecs): recover fps/frame-grid of ms-rounded phone clips | `earlvanze` | `fix/issue-1700-frames-snapped-to-a-1ms-grid` | phone |
| [#1729](https://github.com/storytold/filmcraft/pull/1729) | codecs: the frame rate is what a clip's rounded durations average to (#1700) | `lu123bing` | `fix/mp4-ms-rounded-frame-rate` | mp4 |
| [#1671](https://github.com/storytold/filmcraft/pull/1671) | codecs: start an edited MP4 at its first presented picture | `earlvanze` | `fix/mp4-empty-edit-start-offset` | mp4 |
| [#1662](https://github.com/storytold/filmcraft/pull/1662) | fix(vp9): keep a decoder-pool worker available to the loop filter wavefront (#1283) | `earlvanze` | `fix/vp9-decode-pool-deadlock` | decode, decoder, vp9 |
| [#1648](https://github.com/storytold/filmcraft/pull/1648) | NVENC H.265 export on Linux, and NVENC export at all (#1624) | `ComfyChloe` | `nvenc/hevc-on-linux` | hevc |
| [#1612](https://github.com/storytold/filmcraft/pull/1612) | HW6: VA-API hardware H.264/HEVC export encoding on Linux (#30) | `earlvanze` | `pr408-main-merge` | hevc, h\.?264 |
| [#1580](https://github.com/storytold/filmcraft/pull/1580) | #1221: H.264 High 10 and High 10 Intra export in the built-in encoder | `JordyUSA` | `feat/1221-h264-high10` | encoder, h\.?264 |
| [#1575](https://github.com/storytold/filmcraft/pull/1575) | Media Foundation hardware H.264 / HEVC encoding on Windows (#30) | `ComfyChloe` | `windows/media-foundation-encoder` | encoder, hevc, h\.?264 |
| [#1440](https://github.com/storytold/filmcraft/pull/1440) | #1436: mpegts: H.264/HEVC units are access units; H.264 recovery points are key | `vangel76` | `fix-mpegts-h264-access-units` | hevc, h\.?264 |
| [#1363](https://github.com/storytold/filmcraft/pull/1363) | perf(av1): optimize decoder allocations, zero-fill elimination and DC transforms | `AnarchByte0` | `perf/av1-decoder-optimizations` | decoder, av1 |
| [#1334](https://github.com/storytold/filmcraft/pull/1334) | Stop H.264 export hanging in the YUV conversion | `vbasky` | `fix/1329-yuv-rayon-deadlock` | h\.?264 |
| [#1312](https://github.com/storytold/filmcraft/pull/1312) | Decode YCbCr TIFF stills | `vbasky` | `fix/1293-ycbcr-tiff` | decode |
| [#1289](https://github.com/storytold/filmcraft/pull/1289) | codecs: start an edited MP4 at its first presented picture (#1276) | `bfenty` | `fix/mp4-empty-edit` | mp4 |
| [#1273](https://github.com/storytold/filmcraft/pull/1273) | color: tone-map iPhone HLG footage consistently to SDR | `reoniazx` | `fix/iphone-hlg-sdr` | iphone, color |
| [#1256](https://github.com/storytold/filmcraft/pull/1256) | Linux: VA-API H.264 export encoding (AMD / Intel), on main's `vaapi` module (#30) | `LloydNicholson` | `vaapi-h264-encode` | encode, h\.?264 |
| [#1215](https://github.com/storytold/filmcraft/pull/1215) | fix(timeline): 波形をクリップ背景の暗い同系色で表示する (#1183) | `haranishi` | `fix/1183-waveform-label-color` | color |
| [#1206](https://github.com/storytold/filmcraft/pull/1206) | fix(timeline): 無効クリップのラベル色を保持する (#1190) | `haranishi` | `fix/1190-disabled-clip-color` | color |
| [#1185](https://github.com/storytold/filmcraft/pull/1185) | feat(platform): NVIDIA NVENC hardware AV1 export for MP4 and MOV | `AnarchByte0` | `feat/nvenc-av1-export` | av1, mp4, mov |
| [#1182](https://github.com/storytold/filmcraft/pull/1182) | feat(av1enc): clean-room pure-Rust baseline AV1 software encoder | `AnarchByte0` | `feat/av1enc-software` | encoder, av1 |
| [#1128](https://github.com/storytold/filmcraft/pull/1128) | feat(platform): Intel Quick Sync Video (QSV) hardware encoding for Linux and Windows | `AnarchByte0` | `feat/intel-qsv-encoder` | encoder |
| [#1077](https://github.com/storytold/filmcraft/pull/1077) | #1065: DNG raw settings, faster decode, opcode lists, lossy JPEG DNG | `vangel76` | `dng-decode-plus` | decode |
| [#1076](https://github.com/storytold/filmcraft/pull/1076) | #1039: HEVC decodes pictures on several threads inside (WPP rows, tiles, filters next to parsing) | `vangel76` | `perf-hevc-wpp` | hevc |
| [#993](https://github.com/storytold/filmcraft/pull/993) | #977: MP3 export with a clean-room Layer III encoder | `vangel76` | `gap-mp3-export` | encoder |
| [#884](https://github.com/storytold/filmcraft/pull/884) | h264: safe-SIMD weighted pred, chroma, and IDCT add (#848) | `vbasky` | `feat/848-h264-safe-simd-kernels` | h\.?264 |
| [#883](https://github.com/storytold/filmcraft/pull/883) | h264: safe-SIMD 6-tap luma spike (decoder throughput half of #848) | `vbasky` | `feat/848-h264-safe-simd` | decoder, h\.?264 |
| [#880](https://github.com/storytold/filmcraft/pull/880) | feat(platform): AMD AMF hardware encoding for Linux and Windows | `AnarchByte0` | `feat/amf-and-av1-encoder` | encoder, av1 |
| [#629](https://github.com/storytold/filmcraft/pull/629) | codecs: H.264 High 10 and High 4:2:2 decoding, bit-exact | `4papuce` | `feat/h264-high10-422` | h\.?264 |
| [#409](https://github.com/storytold/filmcraft/pull/409) | AV1 export format, encoded with VA-API on Linux | `mutombopr` | `av1-export-format` | av1 |
| [#408](https://github.com/storytold/filmcraft/pull/408) | Linux: VA-API hardware decoding (H.264, HEVC, VP9, AV1) and H.264 / H.265 export encoding | `mutombopr` | `linux-vaapi` | hevc, h\.?264, av1, vp9 |
| [#352](https://github.com/storytold/filmcraft/pull/352) | Codecs: H.264, HEVC, AAC and ProRes behind cargo features, on by default | `rossaai` | `rossa/optional-encumbered-codecs` | hevc, h\.?264, prores, aac |
| [#152](https://github.com/storytold/filmcraft/pull/152) | Windows: zero-copy hardware decode (D3D11 surface to wgpu DX12) | `SaymonMaster14` | `feat/windows-zero-copy-decode` | decode |

## Category: STABILITY (25 PRs)

| PR | Title | Author | Branch | Matched Keywords |
| :--- | :--- | :--- | :--- | :--- |
| [#2490](https://github.com/storytold/filmcraft/pull/2490) | GPU: bound export read-back waits | `jahvari` | `codex/gpu-bounded-readback` | bound, bounded |
| [#2485](https://github.com/storytold/filmcraft/pull/2485) | AV1: add bounded sequence-header inspection | `jahvari` | `codex/av1-header-inspection` | bounded |
| [#2480](https://github.com/storytold/filmcraft/pull/2480) | fix(mcp): pin doc_inspect's sequence to the reported active id (#1477) | `MichaelMuClaude` | `fix/doc-inspect-sequence-race-1477` | race |
| [#2461](https://github.com/storytold/filmcraft/pull/2461) | #1584: control: a request that timed out is cancelled, never run later | `Felix-Helleckes` | `fix/control-timeout-no-late-run` | timeout |
| [#2394](https://github.com/storytold/filmcraft/pull/2394) | Fix TTML/DFXP export overflow for cue times near i64::MAX | `Silvertree2010` | `fix/h2818-ttml-dfxp-export-panics-with-att` | overflow |
| [#2365](https://github.com/storytold/filmcraft/pull/2365) | fix(engine): timeline.slip by i64::MIN clamps instead of overflowing | `inspiretelapps` | `fix/slip-delta-overflow` | overflow |
| [#2284](https://github.com/storytold/filmcraft/pull/2284) | fix(interchange): a huge duration times a speed saturates instead of overflowing | `Wbunker` | `fix/interchange-scaled-overflow` | overflow |
| [#2282](https://github.com/storytold/filmcraft/pull/2282) | fix(interchange): absurd frame rates and start timecodes are bounded on import | `Wbunker` | `fix/interchange-absurd-frame-rate` | bounded |
| [#2276](https://github.com/storytold/filmcraft/pull/2276) | render: Warp Stabilizer analyses a clip once, in bounded memory | `daveinme` | `fix/stabilizer-analysis-once` | bounded |
| [#2271](https://github.com/storytold/filmcraft/pull/2271) | fix(render): bound graphic effect working rasters | `nuclearspike` | `codex/security-graphic-raster-budget` | bound |
| [#2268](https://github.com/storytold/filmcraft/pull/2268) | fix(render): bound offline placeholder images and cache | `nuclearspike` | `codex/security-offline-slate-budget` | bound |
| [#2136](https://github.com/storytold/filmcraft/pull/2136) | #2024: dialog commands the engine defaults can be bound to a key | `txiki739` | `fix/nest-rebindable` | bound |
| [#2135](https://github.com/storytold/filmcraft/pull/2135) | #1984: silence concealing an MP4 audio packet that will not decode is bounded (its duration is the file's) | `00xJS` | `fix/1984-aac-fallback-allocation` | bounded |
| [#2111](https://github.com/storytold/filmcraft/pull/2111) | Fix potential GPU export deadlock by preparing frame plans outside renderer pool | `ZpdyGonzales` | `fix/gpu-export-deadlock-upstream` | deadlock |
| [#2030](https://github.com/storytold/filmcraft/pull/2030) | fix(shortcuts): Nest… can be bound to a key | `marcoarnulfo` | `fix/nest-bindable` | bound |
| [#1889](https://github.com/storytold/filmcraft/pull/1889) | interchange: avoid char-boundary panic when converting file:// media URLs | `midasmatterhorn` | `fix/file-url-char-boundary` | panic |
| [#1865](https://github.com/storytold/filmcraft/pull/1865) | #1381: the Warp Stabilizer analyses a clip a block of frames at a time | `Guilhermeb-Ferrarezi` | `fix/stabilizer-bounded-memory-1381` | bounded |
| [#1840](https://github.com/storytold/filmcraft/pull/1840) | interchange: overflow-safe FCPXML times and FCP7 timebases, mutation-fuzz harness | `ahmed0magdy` | `contrib/interchange-overflow-fuzz` | overflow |
| [#1808](https://github.com/storytold/filmcraft/pull/1808) | #1061: watchdog for a graphics start that hangs: save the next mode, and the next launch ends the stuck start | `liangforstudy` | `graphics-hang-watchdog` | hang |
| [#1795](https://github.com/storytold/filmcraft/pull/1795) | Hostile-input hardening for video decoders (AV1 panic + oversize allocation, MPEG-2 allocation) and fuzz tests | `g1obalworlds` | `fix/hostile-input-video-codecs` | panic |
| [#1794](https://github.com/storytold/filmcraft/pull/1794) | Hostile-input hammer for container crates; 3 panic fixes | `g1obalworlds` | `fix/hostile-input-containers` | panic |
| [#1662](https://github.com/storytold/filmcraft/pull/1662) | fix(vp9): keep a decoder-pool worker available to the loop filter wavefront (#1283) | `earlvanze` | `fix/vp9-decode-pool-deadlock` | deadlock |
| [#1646](https://github.com/storytold/filmcraft/pull/1646) | feat(updater): crash-resilient self-updating for desktop app | `ARYAN007H` | `main` | crash |
| [#1559](https://github.com/storytold/filmcraft/pull/1559) | export: GPU compositing in the standalone CLI, GPU status in export results, renderer-pool hang (#799) | `bsoulai` | `bsoulai/headless-gpu-799` | hang |
| [#1334](https://github.com/storytold/filmcraft/pull/1334) | Stop H.264 export hanging in the YUV conversion | `vbasky` | `fix/1329-yuv-rayon-deadlock` | deadlock |

