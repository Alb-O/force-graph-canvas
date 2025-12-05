use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

use force_graph::{DefaultNodeIdx, EdgeData, ForceGraph, NodeData, SimulationParameters};

use super::types::GraphData;

const COLORS: &[&str] = &[
	"#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f",
	"#bcbd22", "#17becf",
];

pub const NODE_RADIUS: f64 = 5.0;
pub const HIT_RADIUS: f64 = 12.0;

#[derive(Clone, Debug, Default)]
pub struct NodeInfo {
	pub label: Option<String>,
	pub color: String,
}

#[derive(Clone, Debug, Default)]
pub struct ViewTransform {
	pub x: f64,
	pub y: f64,
	pub k: f64,
}

#[derive(Clone, Debug, Default)]
pub struct DragState {
	pub active: bool,
	pub node_idx: Option<DefaultNodeIdx>,
	pub start_x: f64,
	pub start_y: f64,
	pub node_start_x: f32,
	pub node_start_y: f32,
}

#[derive(Clone, Debug, Default)]
pub struct PanState {
	pub active: bool,
	pub start_x: f64,
	pub start_y: f64,
	pub transform_start_x: f64,
	pub transform_start_y: f64,
}

#[derive(Clone, Debug, Default)]
pub struct HoverState {
	pub node: Option<DefaultNodeIdx>,
	pub neighbors: HashSet<DefaultNodeIdx>,
	pub highlight_t: f64,
	pub prev_node: Option<DefaultNodeIdx>,
	pub prev_neighbors: HashSet<DefaultNodeIdx>,
	delay_t: f64,
}

pub struct ForceGraphState {
	pub graph: ForceGraph<NodeInfo, ()>,
	pub transform: ViewTransform,
	pub drag: DragState,
	pub pan: PanState,
	pub hover: HoverState,
	pub width: f64,
	pub height: f64,
	pub animation_running: bool,
	pub flow_time: f64,
	edges: Vec<(DefaultNodeIdx, DefaultNodeIdx)>,
}

impl ForceGraphState {
	pub fn new(data: &GraphData, width: f64, height: f64) -> Self {
		let mut graph = ForceGraph::new(SimulationParameters {
			force_charge: 150.0,
			force_spring: 0.05,
			force_max: 100.0,
			node_speed: 3000.0,
			damping_factor: 0.9,
		});
		let mut id_to_idx = HashMap::new();
		let mut edges = Vec::new();

		for (i, node) in data.nodes.iter().enumerate() {
			let color = node.color.clone().unwrap_or_else(|| {
				node.group
					.map(|g| COLORS[g as usize % COLORS.len()].into())
					.unwrap_or(COLORS[0].into())
			});
			let angle = (i as f64) * 2.0 * PI / data.nodes.len() as f64;
			let (x, y) = (
				(width / 2.0 + 100.0 * angle.cos()) as f32,
				(height / 2.0 + 100.0 * angle.sin()) as f32,
			);

			let idx = graph.add_node(NodeData {
				x,
				y,
				mass: 10.0,
				is_anchor: false,
				user_data: NodeInfo {
					label: node.label.clone(),
					color,
				},
			});
			id_to_idx.insert(node.id.clone(), idx);
		}

		for link in &data.links {
			if let (Some(&src), Some(&tgt)) =
				(id_to_idx.get(&link.source), id_to_idx.get(&link.target))
			{
				graph.add_edge(src, tgt, EdgeData::default());
				edges.push((src, tgt));
			}
		}

		Self {
			graph,
			edges,
			transform: ViewTransform {
				x: width / 2.0,
				y: height / 2.0,
				k: 1.0,
			},
			drag: DragState::default(),
			pan: PanState::default(),
			hover: HoverState::default(),
			width,
			height,
			animation_running: true,
			flow_time: 0.0,
		}
	}

	pub fn screen_to_graph(&self, sx: f64, sy: f64) -> (f64, f64) {
		(
			(sx - self.transform.x) / self.transform.k,
			(sy - self.transform.y) / self.transform.k,
		)
	}

	pub fn node_at_position(&self, sx: f64, sy: f64) -> Option<DefaultNodeIdx> {
		let (gx, gy) = self.screen_to_graph(sx, sy);
		let mut found = None;
		self.graph.visit_nodes(|node| {
			let (dx, dy) = (node.x() as f64 - gx, node.y() as f64 - gy);
			// HIT_RADIUS is in world-space, scales with zoom like nodes
			if (dx * dx + dy * dy).sqrt() < HIT_RADIUS {
				found = Some(node.index());
			}
		});
		found
	}

	pub fn set_hover(&mut self, node: Option<DefaultNodeIdx>) {
		if self.hover.node == node {
			return;
		}
		let was_hovering = self.hover.node.is_some();

		// Save previous state for fade-out
		if was_hovering && node.is_none() {
			self.hover.prev_node = self.hover.node.take();
			self.hover.prev_neighbors = std::mem::take(&mut self.hover.neighbors);
		} else {
			self.hover.prev_node = None;
			self.hover.prev_neighbors.clear();
		}

		self.hover.node = node;
		self.hover.neighbors.clear();

		if let Some(idx) = node {
			if !was_hovering {
				self.hover.delay_t = 0.0;
			}
			for &(src, tgt) in &self.edges {
				if src == idx {
					self.hover.neighbors.insert(tgt);
				} else if tgt == idx {
					self.hover.neighbors.insert(src);
				}
			}
		}
	}

	pub fn is_highlighted(&self, idx: DefaultNodeIdx) -> bool {
		self.hover.node == Some(idx)
			|| self.hover.neighbors.contains(&idx)
			|| self.hover.prev_node == Some(idx)
			|| self.hover.prev_neighbors.contains(&idx)
	}

	pub fn is_hovered(&self, idx: DefaultNodeIdx) -> bool {
		self.hover.node == Some(idx) || self.hover.prev_node == Some(idx)
	}

	pub fn has_active_highlight(&self) -> bool {
		self.hover.node.is_some() || self.hover.prev_node.is_some()
	}

	pub fn tick(&mut self, dt: f32) {
		self.graph.update(dt);
		self.flow_time += dt as f64;

		let (target, delay, speed) = if self.hover.node.is_some() {
			(1.0, 0.08, 1.8)
		} else {
			(0.0, 0.0, 1.26)
		};

		if self.hover.node.is_some() {
			self.hover.delay_t = (self.hover.delay_t + dt as f64).min(delay);
			if self.hover.delay_t >= delay {
				self.hover.highlight_t += (target - self.hover.highlight_t) * speed * dt as f64;
			}
		} else {
			self.hover.highlight_t += (target - self.hover.highlight_t) * speed * dt as f64;
			if self.hover.highlight_t < 0.01 {
				self.hover.highlight_t = 0.0;
				self.hover.prev_node = None;
				self.hover.prev_neighbors.clear();
			}
		}
	}

	pub fn resize(&mut self, width: f64, height: f64) {
		self.width = width;
		self.height = height;
	}
}
