//! DaVinci Fusion Node Compositor graph and DAG representation.
//!
//! Represents node-based compositing for clips: source inputs (`MediaIn`), procedural
//! processing (ColorGrade, Blur, Glow, Sharpen, Keyer/Matte, Transform), layer blending
//! (`Merge` with foreground, background, and mask inputs), and the terminal `MediaOut`
//! output feeding the sequence timeline.

use std::collections::{BTreeMap, HashMap, VecDeque};

use filmcraft_geom::Vec2;
use serde::{Deserialize, Serialize};

use crate::effect::{EffectInstance, find_effect};
use crate::keyframe::{Param, ParamValue};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub u64);
    };
}
id_type!(NodeId);

/// Sockets on a compositing node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeSocketKind {
    Input,
    Output,
}

/// Node functional types inspired by DaVinci Resolve Fusion.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    /// Ingest source video frames for the current clip.
    MediaIn,
    /// Spatial 2D transform (pan, scale, rotation, anchor).
    Transform,
    /// Lumetri / primary color grading (lift, gamma, gain, sat, contrast, temp, tint).
    ColorGrade,
    /// Gaussian / fast blur filter.
    Blur,
    /// Cinematic bloom / glow filter.
    Glow,
    /// Unsharp mask / edge sharpen filter.
    Sharpen,
    /// Chroma / luma keyer generating an alpha matte.
    Keyer,
    /// Multi-input layer composite (Background, Foreground, Mask) with blend mode & opacity.
    Merge,
    /// Terminal sink that renders the final composite into the timeline track.
    MediaOut,
}

impl NodeType {
    pub fn label(&self) -> &'static str {
        match self {
            NodeType::MediaIn => "MediaIn",
            NodeType::Transform => "Transform",
            NodeType::ColorGrade => "ColorGrade",
            NodeType::Blur => "Blur",
            NodeType::Glow => "Glow",
            NodeType::Sharpen => "Sharpen",
            NodeType::Keyer => "Keyer",
            NodeType::Merge => "Merge",
            NodeType::MediaOut => "MediaOut",
        }
    }

    pub fn default_inputs(&self) -> &'static [&'static str] {
        match self {
            NodeType::MediaIn => &[],
            NodeType::Transform => &["In"],
            NodeType::ColorGrade => &["In"],
            NodeType::Blur => &["In", "Mask"],
            NodeType::Glow => &["In", "Mask"],
            NodeType::Sharpen => &["In"],
            NodeType::Keyer => &["In"],
            NodeType::Merge => &["Background", "Foreground", "Mask"],
            NodeType::MediaOut => &["In"],
        }
    }

    pub fn default_outputs(&self) -> &'static [&'static str] {
        match self {
            NodeType::MediaIn => &["Out"],
            NodeType::Transform => &["Out"],
            NodeType::ColorGrade => &["Out"],
            NodeType::Blur => &["Out"],
            NodeType::Glow => &["Out"],
            NodeType::Sharpen => &["Out"],
            NodeType::Keyer => &["Out", "Matte"],
            NodeType::Merge => &["Out"],
            NodeType::MediaOut => &[],
        }
    }
}

/// A directed connection pipe between two sockets in the DAG.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeConnection {
    pub from_node: NodeId,
    pub from_socket: String,
    pub to_node: NodeId,
    pub to_socket: String,
}

/// An instantiated node in the compositor graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeInstance {
    pub id: NodeId,
    pub name: String,
    pub node_type: NodeType,
    /// Canvas 2D coordinates for the node block.
    pub pos: [f32; 2],
    /// Parameter values (animatable via keyframes).
    pub params: BTreeMap<String, Param>,
    pub enabled: bool,
    /// When true, previewed in Monitor 1 (solo view for raw source or alpha matte).
    #[serde(default)]
    pub solo: bool,
}

impl NodeInstance {
    pub fn new(id: NodeId, name: &str, node_type: NodeType, pos: [f32; 2]) -> Self {
        let mut params = BTreeMap::new();
        match &node_type {
            NodeType::Transform => {
                params.insert("position".into(), Param::new(ParamValue::Vec2(Vec2::new(0.5, 0.5))));
                params.insert("scale".into(), Param::new(ParamValue::Float(100.0)));
                params.insert("rotation".into(), Param::new(ParamValue::Float(0.0)));
            }
            NodeType::ColorGrade => {
                params.insert("exposure".into(), Param::new(ParamValue::Float(0.0)));
                params.insert("contrast".into(), Param::new(ParamValue::Float(0.0)));
                params.insert("saturation".into(), Param::new(ParamValue::Float(100.0)));
                params.insert("temperature".into(), Param::new(ParamValue::Float(0.0)));
            }
            NodeType::Blur => {
                params.insert("radius".into(), Param::new(ParamValue::Float(10.0)));
            }
            NodeType::Glow => {
                params.insert("threshold".into(), Param::new(ParamValue::Float(0.7)));
                params.insert("intensity".into(), Param::new(ParamValue::Float(1.5)));
            }
            NodeType::Sharpen => {
                params.insert("amount".into(), Param::new(ParamValue::Float(25.0)));
            }
            NodeType::Keyer => {
                params.insert("threshold".into(), Param::new(ParamValue::Float(0.15)));
                params.insert("softness".into(), Param::new(ParamValue::Float(0.05)));
            }
            NodeType::Merge => {
                params.insert("opacity".into(), Param::new(ParamValue::Float(100.0)));
                params.insert("blend_mode".into(), Param::new(ParamValue::Choice(0)));
            }
            _ => {}
        }
        Self { id, name: name.to_string(), node_type, pos, params, enabled: true, solo: false }
    }
}

/// Complete Fusion Node Compositor DAG for a clip.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeGraph {
    pub nodes: Vec<NodeInstance>,
    pub connections: Vec<NodeConnection>,
    #[serde(default = "default_next_id")]
    pub next_id: u64,
}

fn default_next_id() -> u64 {
    100
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new_default("Media")
    }
}

impl NodeGraph {
    /// Creates a pristine Fusion compositing graph for a clip:
    /// MediaIn1 -> ColorGrade1 -> MediaOut1
    pub fn new_default(clip_name: &str) -> Self {
        let media_in = NodeInstance::new(NodeId(1), &format!("{clip_name} In"), NodeType::MediaIn, [60.0, 160.0]);
        let grade = NodeInstance::new(NodeId(2), "ColorGrade1", NodeType::ColorGrade, [280.0, 160.0]);
        let media_out = NodeInstance::new(NodeId(3), "MediaOut1", NodeType::MediaOut, [520.0, 160.0]);

        let connections = vec![
            NodeConnection { from_node: NodeId(1), from_socket: "Out".into(), to_node: NodeId(2), to_socket: "In".into() },
            NodeConnection { from_node: NodeId(2), from_socket: "Out".into(), to_node: NodeId(3), to_socket: "In".into() },
        ];

        Self { nodes: vec![media_in, grade, media_out], connections, next_id: 10 }
    }

    /// Converts an existing list of linear effects into a corresponding DAG.
    pub fn from_effects(clip_name: &str, effects: &[EffectInstance]) -> Self {
        let mut nodes = Vec::new();
        let mut connections = Vec::new();

        let mut x = 60.0;
        let y = 160.0;

        let in_id = NodeId(1);
        nodes.push(NodeInstance::new(in_id, &format!("{clip_name} In"), NodeType::MediaIn, [x, y]));

        let mut prev_id = in_id;
        let mut current_id = 2;

        for (i, e) in effects.iter().enumerate() {
            x += 180.0;
            let node_id = NodeId(current_id);
            current_id += 1;

            let ntype = match e.effect.as_str() {
                "blur" | "gaussian_blur" | "fast_blur" => NodeType::Blur,
                "glow" => NodeType::Glow,
                "sharpen" => NodeType::Sharpen,
                "lumetri" | "color" => NodeType::ColorGrade,
                "motion" | "transform" => NodeType::Transform,
                _ => NodeType::ColorGrade,
            };

            let mut node = NodeInstance::new(node_id, &format!("{}_{}", e.effect, i + 1), ntype, [x, y]);
            node.params = e.params.clone();
            node.enabled = e.enabled;
            nodes.push(node);

            connections.push(NodeConnection { from_node: prev_id, from_socket: "Out".into(), to_node: node_id, to_socket: "In".into() });

            prev_id = node_id;
        }

        x += 200.0;
        let out_id = NodeId(current_id);
        nodes.push(NodeInstance::new(out_id, "MediaOut1", NodeType::MediaOut, [x, y]));

        connections.push(NodeConnection { from_node: prev_id, from_socket: "Out".into(), to_node: out_id, to_socket: "In".into() });

        Self { nodes, connections, next_id: current_id + 1 }
    }

    /// Generates equivalent serial EffectInstances for execution in the linear engine pipeline.
    pub fn to_linear_effects(&self) -> Vec<EffectInstance> {
        let order = self.topological_order();
        let mut effects = Vec::new();

        for id in order {
            let Some(node) = self.node(id) else { continue };
            let effect_name = match node.node_type {
                NodeType::Blur => "gaussian_blur",
                NodeType::Glow => "glow",
                NodeType::Sharpen => "sharpen",
                NodeType::ColorGrade => "lumetri",
                NodeType::Transform => "motion",
                NodeType::MediaIn | NodeType::MediaOut | NodeType::Merge | NodeType::Keyer => continue,
            };

            let mut inst = if let Some(def) = find_effect(effect_name) {
                def.instance()
            } else {
                EffectInstance {
                    effect: effect_name.into(),
                    enabled: node.enabled,
                    params: BTreeMap::new(),
                    masks: Vec::new(),
                    post_fader: false,
                    essential: false,
                    layer: None,
                }
            };

            inst.enabled = node.enabled;
            for (k, v) in &node.params {
                inst.params.insert(k.clone(), v.clone());
            }
            effects.push(inst);
        }

        effects
    }

    pub fn node(&self, id: NodeId) -> Option<&NodeInstance> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut NodeInstance> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn add_node(&mut self, name: &str, node_type: NodeType, pos: [f32; 2]) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        self.nodes.push(NodeInstance::new(id, name, node_type, pos));
        id
    }

    pub fn connect(&mut self, from_node: NodeId, from_socket: &str, to_node: NodeId, to_socket: &str) {
        // Disconnect any existing connection to the destination socket (1 input per socket)
        self.disconnect(to_node, to_socket);
        self.connections.push(NodeConnection { from_node, from_socket: from_socket.to_string(), to_node, to_socket: to_socket.to_string() });
    }

    pub fn disconnect(&mut self, to_node: NodeId, to_socket: &str) {
        self.connections.retain(|c| !(c.to_node == to_node && c.to_socket == to_socket));
    }

    /// Computes the evaluation order of the DAG via topological sorting (Kahn's algorithm).
    pub fn topological_order(&self) -> Vec<NodeId> {
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        for n in &self.nodes {
            in_degree.insert(n.id, 0);
            adj.insert(n.id, Vec::new());
        }

        for c in &self.connections {
            if in_degree.contains_key(&c.to_node) && in_degree.contains_key(&c.from_node) {
                *in_degree.entry(c.to_node).or_default() += 1;
                adj.entry(c.from_node).or_default().push(c.to_node);
            }
        }

        let mut queue: VecDeque<NodeId> = self.nodes.iter().map(|n| n.id).filter(|id| in_degree.get(id).copied().unwrap_or(0) == 0).collect();

        let mut order = Vec::new();
        while let Some(u) = queue.pop_front() {
            order.push(u);
            if let Some(neighbors) = adj.get(&u) {
                for &v in neighbors {
                    if let Some(deg) = in_degree.get_mut(&v) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(v);
                        }
                    }
                }
            }
        }

        order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_graph_topological_sort() {
        let graph = NodeGraph::new_default("Clip_01");
        let order = graph.topological_order();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], NodeId(1)); // MediaIn
        assert_eq!(order[1], NodeId(2)); // ColorGrade
        assert_eq!(order[2], NodeId(3)); // MediaOut
    }

    #[test]
    fn test_add_and_connect_merge() {
        let mut graph = NodeGraph::new_default("Clip_01");
        let blur_id = graph.add_node("Blur1", NodeType::Blur, [280.0, 260.0]);
        let merge_id = graph.add_node("Merge1", NodeType::Merge, [400.0, 160.0]);

        graph.disconnect(NodeId(3), "In");
        graph.connect(NodeId(2), "Out", merge_id, "Background");
        graph.connect(blur_id, "Out", merge_id, "Foreground");
        graph.connect(merge_id, "Out", NodeId(3), "In");

        let order = graph.topological_order();
        assert!(order.contains(&blur_id));
        assert!(order.contains(&merge_id));
        assert_eq!(*order.last().unwrap(), NodeId(3));
    }
}
