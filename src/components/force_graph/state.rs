//! Graph simulation state and interaction tracking.
//!
//! Wraps the `force_graph` physics simulation with per-node metadata, view
//! transforms for pan/zoom, and highlight state for hover effects with smooth
//! intensity transitions.

use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

use force_graph::{DefaultNodeIdx, EdgeData, ForceGraph, NodeData, SimulationParameters};

use super::scale::{ScaleConfig, ScaledValues};
use super::theme::Theme;
use super::types::GraphData;

/// Per-node display metadata attached to each node in the simulation.
#[derive(Clone, Debug, Default)]
pub struct NodeInfo {
	pub label: Option<String>,
	pub color: String,
	/// Size multiplier (1.0 = normal, >1.0 = larger/more important)
	pub size: f64,
}

/// Pan and zoom transform applied to the entire graph view.
#[derive(Clone, Debug, Default)]
pub struct ViewTransform {
	pub x: f64,
	pub y: f64,
	/// Zoom factor (1.0 = 100%, clamped to 0.1..10.0).
	pub k: f64,
}

/// Tracks an in-progress node drag operation.
#[derive(Clone, Debug, Default)]
pub struct DragState {
	pub active: bool,
	pub node_idx: Option<DefaultNodeIdx>,
	pub start_x: f64,
	pub start_y: f64,
	pub node_start_x: f32,
	pub node_start_y: f32,
}

/// Tracks an in-progress canvas pan operation.
#[derive(Clone, Debug, Default)]
pub struct PanState {
	pub active: bool,
	pub start_x: f64,
	pub start_y: f64,
	pub transform_start_x: f64,
	pub transform_start_y: f64,
}

/// Manages smooth highlight transitions with per-node intensity tracking.
///
/// Instead of tracking "current" and "previous" highlight sets discretely,
/// each node has its own intensity value (0.0 to 1.0) that smoothly animates
/// based on whether it's in the active highlight set.
///
/// Uses exponential smoothing for natural-feeling transitions that slow down
/// as they approach their target (like spring physics).
///
/// Includes a minimum hold time to prevent flashing when the mouse briefly
/// skirts the edge of a node's hover zone.
#[derive(Clone, Debug, Default)]
pub struct HighlightState {
	/// Currently hovered node (if any)
	pub hovered_node: Option<DefaultNodeIdx>,
	/// Set of nodes that should be highlighted (hovered + neighbors)
	target_set: HashSet<DefaultNodeIdx>,
	/// Per-node highlight intensity (0.0 = not highlighted, 1.0 = fully highlighted)
	/// Nodes not in this map have intensity 0.
	node_intensity: HashMap<DefaultNodeIdx, f64>,
	/// Smoothed hover intensity for the ring effect (tracks hovered_node with hold time)
	hover_ring_intensity: HashMap<DefaultNodeIdx, f64>,
	/// Per-node hold timer - time remaining before fade-out can begin
	hold_timer: HashMap<DefaultNodeIdx, f64>,
	/// Cached max intensity (updated each tick)
	cached_max: f64,
}

/// Minimum time (seconds) a highlight must be held before it can fade out.
/// This prevents flashing when the mouse briefly touches a hover zone.
const MIN_HOLD_TIME: f64 = 0.12;

impl HighlightState {
	/// Update the hovered node and recompute the target highlight set.
	pub fn set_hover(
		&mut self,
		node: Option<DefaultNodeIdx>,
		edges: &[(DefaultNodeIdx, DefaultNodeIdx)],
	) {
		if self.hovered_node == node {
			return;
		}

		self.hovered_node = node;
		self.target_set.clear();

		if let Some(idx) = node {
			// Add hovered node
			self.target_set.insert(idx);
			// Add neighbors
			for &(src, tgt) in edges {
				if src == idx {
					self.target_set.insert(tgt);
				} else if tgt == idx {
					self.target_set.insert(src);
				}
			}

			// Reset hold timers for newly highlighted nodes
			for &idx in &self.target_set {
				self.hold_timer.insert(idx, MIN_HOLD_TIME);
			}
		}
	}

	/// Animate all node intensities towards their targets using exponential smoothing.
	///
	/// Exponential smoothing: value += (target - value) * (1 - e^(-speed * dt))
	/// This creates natural ease-out behavior where animation slows as it approaches target.
	pub fn tick(&mut self, dt: f64) {
		// Smoothing factors - higher = faster response
		// At 60fps with speed=8: reaches ~63% in ~2 frames, ~95% in ~6 frames (~100ms)
		// At 60fps with speed=5: reaches ~63% in ~4 frames, ~95% in ~12 frames (~200ms)
		const FADE_IN_SPEED: f64 = 6.0; // ~150ms to 95%
		const FADE_OUT_SPEED: f64 = 4.0; // ~250ms to 95%

		let fade_in_factor = 1.0 - (-FADE_IN_SPEED * dt).exp();
		let fade_out_decay = (-FADE_OUT_SPEED * dt).exp();

		// Animate nodes in target set (fade in)
		for &idx in &self.target_set {
			let intensity = self.node_intensity.entry(idx).or_insert(0.0);
			// Exponential smoothing towards 1.0
			*intensity += (1.0 - *intensity) * fade_in_factor;
		}

		// Animate hover ring intensity (only for the hovered node)
		if let Some(idx) = self.hovered_node {
			let intensity = self.hover_ring_intensity.entry(idx).or_insert(0.0);
			*intensity += (1.0 - *intensity) * fade_in_factor;
		}

		// Track max for caching
		let mut new_max: f64 = 0.0;

		// Update hold timers and animate fade-out
		self.hold_timer.retain(|idx, timer| {
			if self.target_set.contains(idx) {
				// Node is still highlighted, keep the timer
				true
			} else {
				// Node is no longer in target set, count down
				*timer -= dt;
				*timer > 0.0
			}
		});

		// Animate nodes not in target set (fade out) and remove when done
		self.node_intensity.retain(|idx, intensity| {
			if self.target_set.contains(idx) {
				new_max = new_max.max(*intensity);
				true
			} else {
				// Only fade out if hold timer has expired
				let hold_remaining = self.hold_timer.get(idx).copied().unwrap_or(0.0);
				if hold_remaining <= 0.0 {
					// Exponential decay towards 0.0
					*intensity *= fade_out_decay;
				}
				new_max = new_max.max(*intensity);
				*intensity > 0.005 // Keep only if still visible
			}
		});

		// Animate hover ring fade-out (respects hold timer)
		self.hover_ring_intensity.retain(|idx, intensity| {
			if self.hovered_node == Some(*idx) {
				true // Still hovered, keep at current intensity
			} else {
				// Only fade out if hold timer has expired
				let hold_remaining = self.hold_timer.get(idx).copied().unwrap_or(0.0);
				if hold_remaining <= 0.0 {
					*intensity *= fade_out_decay;
				}
				*intensity > 0.005
			}
		});

		self.cached_max = new_max;
	}

	/// Get the highlight intensity for a specific node (already smoothed).
	pub fn node_intensity(&self, idx: DefaultNodeIdx) -> f64 {
		self.node_intensity.get(&idx).copied().unwrap_or(0.0)
	}

	/// Get the hover ring intensity for a specific node (smoothed, with hold time).
	pub fn hover_ring_intensity(&self, idx: DefaultNodeIdx) -> f64 {
		self.hover_ring_intensity.get(&idx).copied().unwrap_or(0.0)
	}

	/// Get the highlight intensity for an edge.
	/// Uses geometric mean for smoother edge transitions that don't lag behind nodes.
	pub fn edge_intensity(&self, idx1: DefaultNodeIdx, idx2: DefaultNodeIdx) -> f64 {
		let i1 = self.node_intensity(idx1);
		let i2 = self.node_intensity(idx2);
		// Geometric mean is smoother than min for transitions
		(i1 * i2).sqrt()
	}

	/// Get the maximum intensity of any node (useful for dimming non-highlighted elements).
	pub fn max_intensity(&self) -> f64 {
		self.cached_max
	}
}

/// Core graph state combining physics simulation with interaction and highlight tracking.
///
/// Created once when the component mounts, then mutated each frame by the
/// animation loop. The `tick` method advances the physics simulation and
/// animates highlight intensities.
pub struct ForceGraphState {
	pub graph: ForceGraph<NodeInfo, ()>,
	pub transform: ViewTransform,
	pub drag: DragState,
	pub pan: PanState,
	pub highlight: HighlightState,
	pub width: f64,
	pub height: f64,
	pub animation_running: bool,
	pub flow_time: f64,
	edges: Vec<(DefaultNodeIdx, DefaultNodeIdx)>,
}

impl ForceGraphState {
	pub fn new(data: &GraphData, width: f64, height: f64, theme: &Theme) -> Self {
		let mut graph = ForceGraph::new(SimulationParameters {
			force_charge: 150.0,
			force_spring: 0.05,
			force_max: 100.0,
			node_speed: 3000.0,
			damping_factor: 0.9,
		});
		let mut id_to_idx = HashMap::new();
		let mut edges = Vec::new();

		// Count edges per node for importance calculation
		let mut edge_counts: HashMap<&String, usize> = HashMap::new();
		for link in &data.links {
			*edge_counts.entry(&link.source).or_insert(0) += 1;
			*edge_counts.entry(&link.target).or_insert(0) += 1;
		}
		let max_edges = edge_counts.values().copied().max().unwrap_or(1).max(1);

		for (i, node) in data.nodes.iter().enumerate() {
			let color = node.color.clone().unwrap_or_else(|| {
				node.group
					.map(|g| theme.palette.get(g as usize).to_css_rgb())
					.unwrap_or_else(|| theme.palette.get(i).to_css_rgb())
			});
			let angle = (i as f64) * 2.0 * PI / data.nodes.len() as f64;
			let (x, y) = (
				(width / 2.0 + 100.0 * angle.cos()) as f32,
				(height / 2.0 + 100.0 * angle.sin()) as f32,
			);

			// Calculate node importance/size based on:
			// - Having a label (more important)
			// - Number of connections (more connected = larger)
			let has_label = node.label.is_some();
			let node_edges = edge_counts.get(&node.id).copied().unwrap_or(0);
			let edge_factor = (node_edges as f64 / max_edges as f64).sqrt(); // sqrt for softer scaling

			let size = if has_label {
				1.4 + 0.6 * edge_factor // labeled: 1.4x to 2.0x
			} else {
				0.7 + 0.5 * edge_factor // unlabeled: 0.7x to 1.2x
			};

			let idx = graph.add_node(NodeData {
				x,
				y,
				mass: 10.0,
				is_anchor: false,
				user_data: NodeInfo {
					label: node.label.clone(),
					color,
					size,
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
			highlight: HighlightState::default(),
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

	pub fn node_at_position(
		&self,
		sx: f64,
		sy: f64,
		config: &ScaleConfig,
	) -> Option<DefaultNodeIdx> {
		let (gx, gy) = self.screen_to_graph(sx, sy);
		let scale = ScaledValues::new(config, self.transform.k);
		let mut found = None;
		self.graph.visit_nodes(|node| {
			let (dx, dy) = (node.x() as f64 - gx, node.y() as f64 - gy);
			let node_hit_radius = scale.hit_radius * node.data.user_data.size;
			if (dx * dx + dy * dy).sqrt() < node_hit_radius {
				found = Some(node.index());
			}
		});
		found
	}

	pub fn set_hover(&mut self, node: Option<DefaultNodeIdx>) {
		self.highlight.set_hover(node, &self.edges);
	}

	pub fn tick(&mut self, dt: f32) {
		self.graph.update(dt);
		self.flow_time += dt as f64;
		self.highlight.tick(dt as f64);
	}

	pub fn resize(&mut self, width: f64, height: f64) {
		self.width = width;
		self.height = height;
	}
}
