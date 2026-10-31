//! Bounded, device-free frame-plan messages for independent browser WASM workers.
//!
//! Metadata is small JSON; pixels and already-prepared upload planes are transferable
//! binary buffers. No GPU device, browser Blob, pointer or WASM memory crosses the boundary.
//! Acknowledged immutable frames keep their identity across parameter edits, avoiding
//! repeated pixel copies and texture uploads. The receiver owns its bounded cache.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Weak};

use filmcraft_color::{ColorInfo, InputTransform};
use filmcraft_frame::{Chroma, PixelData, VideoFrame};
use filmcraft_geom::Affine;
use filmcraft_render::Blend;
use filmcraft_render::plan::{FramePlan, LayerFx, PlanLayer, PlanStep};
use filmcraft_time::Tick;
use serde::{Deserialize, Serialize};

use crate::{Prepared, PreparedPlan};

pub const MAX_PACKET_BYTES: usize = 256 << 20;
const MAX_METADATA: usize = 1 << 20;
const MAX_FRAMES: usize = 256;
const CACHE_BYTES: usize = 128 << 20;
const MAX_SIDE: usize = 16_384;

pub struct Packet {
    pub metadata: String,
    pub buffers: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    version: u8,
    plan: WirePlan,
    frames: Vec<WireFrame>,
}

#[derive(Serialize, Deserialize)]
enum WirePlan {
    Layers { width: usize, height: usize, layers: Vec<WireLayer> },
    Composite { width: usize, height: usize, steps: Vec<WireStep> },
}

#[derive(Serialize, Deserialize)]
enum WireStep {
    Layers(Vec<WireLayer>),
    Transition { inputs: [Vec<WireLayer>; 2], effect: filmcraft_project::EffectInstance, progress: f32, scale: f32 },
}

#[derive(Serialize, Deserialize)]
struct WireLayer {
    frame: u32,
    matrix: Affine,
    opacity: f32,
    blend: Blend,
    fx: Option<Arc<LayerFx>>,
    input: Option<Arc<InputTransform>>,
}

#[derive(Serialize, Deserialize)]
struct WireFrame {
    id: u32,
    width: u32,
    height: u32,
    color: ColorInfo,
    par: (u32, u32),
    pts: Tick,
    pixels: Pixels,
    prepared: Option<Vec<usize>>,
}

#[derive(Serialize, Deserialize)]
enum Pixels {
    Rgba8(usize),
    RgbaF32(usize),
    Yuv8 { planes: [usize; 3], chroma: Chroma, alpha: Option<usize> },
    Yuv16 { planes: [usize; 3], chroma: Chroma, bits: u32, alpha: Option<usize> },
}

#[derive(Default)]
pub struct Encoder {
    next_id: u32,
    frames: HashMap<usize, (Weak<VideoFrame>, u32)>,
}

struct Writer<'a> {
    encoder: &'a mut Encoder,
    acknowledged: HashSet<u32>,
    sent: HashSet<u32>,
    frames: Vec<WireFrame>,
    buffers: Vec<Vec<u8>>,
    bytes: usize,
}

fn dimensions(w: usize, h: usize) -> Result<usize, String> {
    if w == 0 || h == 0 || w > MAX_SIDE || h > MAX_SIDE {
        return Err("frame transport dimensions outside supported bounds".into());
    }
    w.checked_mul(h).filter(|n| *n <= MAX_PACKET_BYTES / 4).ok_or_else(|| "frame transport dimensions overflow".into())
}

impl Encoder {
    pub fn encode(&mut self, plan: &FramePlan, prepared: &PreparedPlan, retained_frames: &[u32]) -> Result<Packet, String> {
        self.frames.retain(|_, (f, _)| f.strong_count() > 0);
        if retained_frames.len() > MAX_FRAMES {
            return Err("too many retained frame references".into());
        }
        let mut writer = Writer {
            encoder: self,
            acknowledged: retained_frames.iter().copied().collect(),
            sent: HashSet::new(),
            frames: Vec::new(),
            buffers: Vec::new(),
            bytes: 0,
        };
        let wire = match plan {
            FramePlan::Layers { width, height, layers } => {
                dimensions(*width, *height)?;
                WirePlan::Layers { width: *width, height: *height, layers: writer.layers(layers, prepared)? }
            }
            FramePlan::Composite { width, height, steps } => {
                dimensions(*width, *height)?;
                if steps.len() > MAX_FRAMES {
                    return Err("too many composite steps".into());
                }
                let mut children = prepared.children.iter();
                let mut out = Vec::new();
                for step in steps {
                    out.push(match step {
                        PlanStep::Layers(layers) => WireStep::Layers(writer.layers(layers, children.next().ok_or("missing prepared composite child")?)?),
                        PlanStep::Transition { inputs, effect, progress, scale } => WireStep::Transition {
                            inputs: [
                                writer.layers(&inputs[0], children.next().ok_or("missing prepared transition input")?)?,
                                writer.layers(&inputs[1], children.next().ok_or("missing prepared transition input")?)?,
                            ],
                            effect: effect.clone(),
                            progress: *progress,
                            scale: *scale,
                        },
                    });
                }
                WirePlan::Composite { width: *width, height: *height, steps: out }
            }
            FramePlan::Image(img) => {
                dimensions(img.w, img.h)?;
                if img.px.len() != img.w.saturating_mul(img.h).saturating_mul(4) {
                    return Err("invalid fallback image".into());
                }
                // The compositor itself represents Image as this identity layer. Do the
                // same here to share its raw float buffer without a UI-side deep clone.
                let frame = Arc::new(VideoFrame::rgba_f32(img.w as u32, img.h as u32, img.px.clone()));
                let id = writer.frame(&frame, prepared.image.as_ref())?;
                WirePlan::Layers {
                    width: img.w,
                    height: img.h,
                    layers: vec![WireLayer { frame: id, matrix: Affine::IDENTITY, opacity: 1.0, blend: Blend::Normal, fx: None, input: None }],
                }
            }
        };
        let metadata = serde_json::to_string(&Message { version: 1, plan: wire, frames: writer.frames }).map_err(|e| e.to_string())?;
        if metadata.len() > MAX_METADATA {
            return Err("frame plan metadata exceeds limit".into());
        }
        Ok(Packet { metadata, buffers: writer.buffers })
    }
}

impl Writer<'_> {
    fn buffer(&mut self, bytes: Vec<u8>) -> Result<usize, String> {
        self.bytes = self.bytes.checked_add(bytes.len()).ok_or("frame packet size overflow")?;
        if self.bytes > MAX_PACKET_BYTES || self.buffers.len() >= MAX_FRAMES * 8 {
            return Err("frame packet exceeds transfer budget".into());
        }
        let id = self.buffers.len();
        self.buffers.push(bytes);
        Ok(id)
    }

    fn frame(&mut self, frame: &Arc<VideoFrame>, prepared: Option<&Prepared>) -> Result<u32, String> {
        dimensions(frame.width as usize, frame.height as usize)?;
        let pointer = Arc::as_ptr(frame) as usize;
        let id = match self.encoder.frames.get(&pointer).filter(|(f, _)| f.upgrade().is_some_and(|f| Arc::ptr_eq(&f, frame))) {
            Some((_, id)) => *id,
            None => {
                self.encoder.next_id = self.encoder.next_id.checked_add(1).ok_or("frame identity exhausted; restart renderer")?;
                let id = self.encoder.next_id;
                self.encoder.frames.insert(pointer, (Arc::downgrade(frame), id));
                id
            }
        };
        if self.acknowledged.contains(&id) || !self.sent.insert(id) {
            return Ok(id);
        }
        if self.frames.len() >= MAX_FRAMES {
            return Err("too many transported frames".into());
        }
        let outgoing = frame.byte_size().checked_add(prepared.map_or(0, Prepared::bytes)).ok_or("frame byte size overflow")?;
        if outgoing > MAX_PACKET_BYTES.saturating_sub(self.bytes) {
            return Err("frame exceeds remaining transfer budget".into());
        }
        let mut u16s = |v: &[u16]| self.buffer(v.iter().flat_map(|v| v.to_le_bytes()).collect());
        let pixels = match &frame.data {
            PixelData::Rgba8(v) => Pixels::Rgba8(self.buffer(v.as_ref().clone())?),
            PixelData::RgbaF32(v) => Pixels::RgbaF32(self.buffer(v.iter().flat_map(|v| v.to_le_bytes()).collect())?),
            PixelData::Yuv8 { planes, chroma, alpha } => Pixels::Yuv8 {
                planes: [self.buffer(planes[0].as_ref().clone())?, self.buffer(planes[1].as_ref().clone())?, self.buffer(planes[2].as_ref().clone())?],
                chroma: *chroma,
                alpha: alpha.as_ref().map(|a| self.buffer(a.as_ref().clone())).transpose()?,
            },
            PixelData::Yuv16 { planes, chroma, bits, alpha } => Pixels::Yuv16 {
                planes: [u16s(&planes[0])?, u16s(&planes[1])?, u16s(&planes[2])?],
                chroma: *chroma,
                bits: *bits,
                alpha: alpha.as_ref().map(|a| u16s(a)).transpose()?,
            },
        };
        let prepared = prepared.map(|p| p.planes.iter().map(|v| self.buffer(v.as_ref().clone())).collect::<Result<Vec<_>, _>>()).transpose()?;
        self.frames.push(WireFrame { id, width: frame.width, height: frame.height, color: frame.color, par: frame.par, pts: frame.pts, pixels, prepared });
        Ok(id)
    }

    fn layers(&mut self, layers: &[PlanLayer], prepared: &PreparedPlan) -> Result<Vec<WireLayer>, String> {
        if layers.len() > MAX_FRAMES || prepared.layers.len() != layers.len() {
            return Err("invalid prepared layer count".into());
        }
        layers
            .iter()
            .zip(&prepared.layers)
            .map(|(l, p)| {
                Ok(WireLayer {
                    frame: self.frame(&l.frame, p.as_ref())?,
                    matrix: l.matrix,
                    opacity: l.opacity,
                    blend: l.blend,
                    fx: l.fx.clone(),
                    input: l.input.clone(),
                })
            })
            .collect()
    }
}

struct CachedFrame {
    frame: Arc<VideoFrame>,
    prepared: Option<Prepared>,
    bytes: usize,
    used: u64,
}

#[derive(Default)]
pub struct Decoder {
    frames: HashMap<u32, CachedFrame>,
    clock: u64,
    bytes: usize,
}

impl Decoder {
    pub fn retained_frames(&self) -> Vec<u32> {
        self.frames.keys().copied().collect()
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn decode(&mut self, packet: Packet) -> Result<(FramePlan, PreparedPlan), String> {
        let result = self.decode_inner(packet);
        // Rejected packets must not bypass eviction and accumulate retained frames.
        while self.bytes > CACHE_BYTES || self.frames.len() > 96 {
            let Some(id) = self.frames.iter().min_by_key(|(_, f)| f.used).map(|(id, _)| *id) else { break };
            if let Some(f) = self.frames.remove(&id) {
                self.bytes = self.bytes.saturating_sub(f.bytes);
            }
        }
        result
    }

    fn decode_inner(&mut self, mut packet: Packet) -> Result<(FramePlan, PreparedPlan), String> {
        if packet.metadata.len() > MAX_METADATA
            || packet.buffers.len() > MAX_FRAMES * 8
            || packet.buffers.iter().try_fold(0usize, |n, b| n.checked_add(b.len())).is_none_or(|n| n > MAX_PACKET_BYTES)
        {
            return Err("received frame packet exceeds budget".into());
        }
        let message: Message = serde_json::from_str(&packet.metadata).map_err(|e| format!("invalid frame metadata: {e}"))?;
        if message.version != 1 || message.frames.len() > MAX_FRAMES {
            return Err("unsupported frame packet".into());
        }
        self.clock = self.clock.saturating_add(1);
        let mut ids = HashSet::new();
        for f in message.frames {
            if !ids.insert(f.id) {
                return Err("duplicate transported frame identity".into());
            }
            let frame = decode_frame(&f, &mut packet.buffers)?;
            let sizes = prepared_sizes(&frame);
            let prepared = match (&f.prepared, sizes) {
                (None, None) => None,
                (Some(indices), Some(sizes)) if indices.len() == sizes.len() => Some(Prepared {
                    planes: indices.iter().zip(sizes).map(|(i, n)| take_buffer(&mut packet.buffers, *i, n).map(Arc::new)).collect::<Result<_, _>>()?,
                }),
                _ => return Err("missing or invalid prepared upload planes".into()),
            };
            let bytes = frame.byte_size().checked_add(prepared.as_ref().map_or(0, Prepared::bytes)).ok_or("frame size overflow")?;
            let entry = CachedFrame { frame: Arc::new(frame), prepared, bytes, used: self.clock };
            if let Some(old) = self.frames.insert(f.id, entry) {
                self.bytes = self.bytes.saturating_sub(old.bytes);
            }
            self.bytes = self.bytes.saturating_add(bytes);
        }
        let result = match message.plan {
            WirePlan::Layers { width, height, layers } => {
                dimensions(width, height)?;
                let (layers, prepared) = self.layers(layers)?;
                (FramePlan::Layers { width, height, layers }, prepared)
            }
            WirePlan::Composite { width, height, steps } => {
                dimensions(width, height)?;
                if steps.len() > MAX_FRAMES {
                    return Err("too many received composite steps".into());
                }
                let mut out = Vec::new();
                let mut prepared = PreparedPlan::default();
                for s in steps {
                    out.push(match s {
                        WireStep::Layers(ls) => {
                            let (ls, p) = self.layers(ls)?;
                            prepared.children.push(p);
                            PlanStep::Layers(ls)
                        }
                        WireStep::Transition { inputs, effect, progress, scale } => {
                            let [a, b] = inputs;
                            let (a, pa) = self.layers(a)?;
                            let (b, pb) = self.layers(b)?;
                            prepared.children.extend([pa, pb]);
                            PlanStep::Transition { inputs: [a, b], effect, progress, scale }
                        }
                    });
                }
                (FramePlan::Composite { width, height, steps: out }, prepared)
            }
        };
        Ok(result)
    }

    fn layers(&mut self, input: Vec<WireLayer>) -> Result<(Vec<PlanLayer>, PreparedPlan), String> {
        if input.len() > MAX_FRAMES {
            return Err("too many received layers".into());
        }
        let mut layers = Vec::new();
        let mut prepared = PreparedPlan::default();
        for l in input {
            if l.fx.as_ref().is_some_and(|fx| fx.ops.len() > 4096 || dimensions(fx.size.0 as usize, fx.size.1 as usize).is_err()) {
                return Err("invalid effect working image".into());
            }
            let f = self.frames.get_mut(&l.frame).ok_or("unknown retained frame; resend pixels")?;
            f.used = self.clock;
            layers.push(PlanLayer { frame: f.frame.clone(), matrix: l.matrix, opacity: l.opacity, blend: l.blend, fx: l.fx, input: l.input });
            prepared.layers.push(f.prepared.clone());
        }
        Ok((layers, prepared))
    }
}

fn take_buffer(buffers: &mut [Vec<u8>], id: usize, length: usize) -> Result<Vec<u8>, String> {
    let b = buffers.get_mut(id).filter(|b| b.len() == length && length > 0).ok_or("invalid or repeated pixel plane reference")?;
    Ok(std::mem::take(b))
}

fn prepared_sizes(frame: &VideoFrame) -> Option<Vec<usize>> {
    let n = frame.width as usize * frame.height as usize;
    match &frame.data {
        PixelData::RgbaF32(_) => Some(vec![n * 8]),
        PixelData::Yuv16 { planes, alpha, .. } => Some(planes.iter().chain(alpha).map(|p| p.len() * 2).collect()),
        _ => None,
    }
}

fn decode_frame(f: &WireFrame, buffers: &mut [Vec<u8>]) -> Result<VideoFrame, String> {
    let n = dimensions(f.width as usize, f.height as usize)?;
    if f.par.0 == 0 || f.par.1 == 0 {
        return Err("invalid transported pixel aspect ratio".into());
    }
    fn u16s(buffers: &mut [Vec<u8>], id: usize, n: usize) -> Result<Arc<Vec<u16>>, String> {
        Ok(Arc::new(take_buffer(buffers, id, n * 2)?.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).collect()))
    }
    let chroma_len = |chroma: Chroma| {
        let (sx, sy) = chroma.shifts();
        (f.width as usize).div_ceil(1 << sx) * (f.height as usize).div_ceil(1 << sy)
    };
    let data = match &f.pixels {
        Pixels::Rgba8(id) => PixelData::Rgba8(Arc::new(take_buffer(buffers, *id, n * 4)?)),
        Pixels::RgbaF32(id) => {
            PixelData::RgbaF32(Arc::new(take_buffer(buffers, *id, n * 16)?.as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c)).collect()))
        }
        Pixels::Yuv8 { planes, chroma, alpha } => {
            let c = chroma_len(*chroma);
            PixelData::Yuv8 {
                planes: [
                    Arc::new(take_buffer(buffers, planes[0], n)?),
                    Arc::new(take_buffer(buffers, planes[1], c)?),
                    Arc::new(take_buffer(buffers, planes[2], c)?),
                ],
                chroma: *chroma,
                alpha: alpha.map(|i| take_buffer(buffers, i, n).map(Arc::new)).transpose()?,
            }
        }
        Pixels::Yuv16 { planes, chroma, bits, alpha } => {
            if !(9..=16).contains(bits) {
                return Err("invalid transported bit depth".into());
            }
            let c = chroma_len(*chroma);
            PixelData::Yuv16 {
                planes: [u16s(buffers, planes[0], n)?, u16s(buffers, planes[1], c)?, u16s(buffers, planes[2], c)?],
                chroma: *chroma,
                bits: *bits,
                alpha: alpha.map(|i| u16s(buffers, i, n)).transpose()?,
            }
        }
    };
    Ok(VideoFrame { width: f.width, height: f.height, color: f.color, par: f.par, pts: f.pts, data })
}

#[cfg(test)]
mod tests {
    use super::*;
    use filmcraft_render::gpufx::FxOp;
    use filmcraft_render::plan::execute_cpu;

    fn plan(frame: Arc<VideoFrame>) -> FramePlan {
        let mut layer = PlanLayer::new(frame, Affine::IDENTITY, 0.75, Blend::Normal);
        layer.fx =
            Some(Arc::new(LayerFx { size: (7, 5), decimation: 1, ops: vec![FxOp::WaveWarp { height: 1.5, wavelength: 6.0, ux: 1.0, uy: 0.0, phase: 0.37 }] }));
        FramePlan::Layers { width: 7, height: 5, layers: vec![layer] }
    }

    #[test]
    fn binary_planes_and_prepared_uploads_round_trip_without_color_or_time_changes() {
        let source = [
            PixelData::Rgba8(Arc::new((0..140).map(|i| i as u8).collect())),
            PixelData::RgbaF32(Arc::new((0..140).map(|i| i as f32 / 70.0).collect())),
            PixelData::Yuv8 {
                planes: [Arc::new(vec![100; 35]), Arc::new(vec![80; 20]), Arc::new(vec![160; 20])],
                chroma: Chroma::C422,
                alpha: Some(Arc::new(vec![180; 35])),
            },
            PixelData::Yuv16 {
                planes: [Arc::new(vec![700; 35]), Arc::new(vec![400; 12]), Arc::new(vec![600; 12])],
                chroma: Chroma::C420,
                bits: 10,
                alpha: Some(Arc::new(vec![900; 35])),
            },
        ];
        for data in source {
            let frame = Arc::new(VideoFrame { width: 7, height: 5, data, color: ColorInfo::SRGB_FULL, par: (4, 3), pts: Tick(i64::MAX / 4) });
            let plan = plan(frame);
            let prepared = crate::prepare(&plan);
            let mut encoder = Encoder::default();
            let mut decoder = Decoder::default();
            let packet = encoder.encode(&plan, &prepared, &[]).unwrap();
            assert!(packet.metadata.len() < 4096, "pixels must not be encoded as JSON numbers");
            let (got, upload) = decoder.decode(packet).unwrap();
            assert_eq!(execute_cpu(&got).px, execute_cpu(&plan).px);
            assert_eq!(prepared.bytes(), upload.bytes());
            let FramePlan::Layers { layers, .. } = &got else { panic!("layers") };
            assert_eq!(layers[0].frame.par, (4, 3));
            assert_eq!(layers[0].frame.pts, Tick(i64::MAX / 4), "exact integer ticks survive the message");
            let packet = encoder.encode(&plan, &prepared, &decoder.retained_frames()).unwrap();
            assert!(packet.buffers.is_empty(), "unchanged source pixels must not be recopied on parameter edits");
            let (again, _) = decoder.decode(packet).unwrap();
            let FramePlan::Layers { layers: again, .. } = again else { panic!("layers") };
            assert!(Arc::ptr_eq(&layers[0].frame, &again[0].frame), "preserve GPU upload identity");
        }
    }

    #[test]
    fn fallback_images_and_ordered_transitions_keep_the_cpu_picture() {
        let image = filmcraft_render::Image::filled(7, 5, [0.2, 0.1, 0.3, 0.5]);
        let image_plan = FramePlan::Image(image);
        let layer = PlanLayer::new(Arc::new(VideoFrame::rgba8(7, 5, vec![128; 140])), Affine::IDENTITY, 0.8, Blend::Normal);
        let composite = FramePlan::Composite {
            width: 7,
            height: 5,
            steps: vec![PlanStep::Transition {
                inputs: [vec![layer.clone()], vec![layer]],
                effect: filmcraft_project::find_effect("push").unwrap().instance(),
                progress: 0.375,
                scale: 1.0,
            }],
        };
        for plan in [image_plan, composite] {
            let mut encoder = Encoder::default();
            let mut decoder = Decoder::default();
            let (got, _) = decoder.decode(encoder.encode(&plan, &crate::prepare(&plan), &[]).unwrap()).unwrap();
            assert_eq!(execute_cpu(&got).px, execute_cpu(&plan).px);
        }
    }

    #[test]
    fn missing_references_malformed_planes_and_dimensions_return_errors() {
        let frame = Arc::new(VideoFrame::rgba8(7, 5, vec![0; 140]));
        let plan = plan(frame);
        let prepared = crate::prepare(&plan);
        let mut encoder = Encoder::default();
        let mut packet = encoder.encode(&plan, &prepared, &[]).unwrap();
        packet.buffers[0].truncate(1);
        assert!(Decoder::default().decode(packet).is_err());
        let packet = encoder.encode(&plan, &prepared, &[1]).unwrap();
        assert!(Decoder::default().decode(packet).is_err());
        let mut packet = encoder.encode(&plan, &prepared, &[]).unwrap();
        let mut m: Message = serde_json::from_str(&packet.metadata).unwrap();
        m.frames[0].width = u32::MAX;
        packet.metadata = serde_json::to_string(&m).unwrap();
        assert!(Decoder::default().decode(packet).is_err());
        let packet = Packet { metadata: "{".into(), buffers: vec![] };
        assert!(Decoder::default().decode(packet).is_err());
    }

    #[test]
    fn transport_dictionary_is_bounded_and_evicted_frames_can_be_resent() {
        let mut encoder = Encoder::default();
        let mut decoder = Decoder::default();
        let mut first = None;
        for n in 0..110 {
            let p = plan(Arc::new(VideoFrame::rgba8(7, 5, vec![n as u8; 140])));
            if first.is_none() {
                first = Some(p.clone());
            }
            decoder.decode(encoder.encode(&p, &crate::prepare(&p), &decoder.retained_frames()).unwrap()).unwrap();
            assert!(decoder.retained_frames().len() <= 96);
            assert!(decoder.bytes() <= CACHE_BYTES);
        }
        let p = first.unwrap();
        let (got, _) = decoder.decode(encoder.encode(&p, &crate::prepare(&p), &decoder.retained_frames()).unwrap()).unwrap();
        assert_eq!(execute_cpu(&got).px, execute_cpu(&p).px);
    }
}
