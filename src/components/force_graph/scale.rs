//! Zoom-dependent scaling configuration for graph visuals.
//!
//! This module centralizes all zoom-dependent visual parameters, making it easy to
//! understand and tune how elements behave at different zoom levels.
//!
//! # Coordinate Spaces
//!
//! - **World-space**: The coordinate system of the graph. Values in world-space
//!   scale proportionally with zoom (appear larger when zoomed in).
//! - **Screen-space**: Pixel coordinates on the canvas. Values in screen-space
//!   remain constant regardless of zoom level.
//!
//! # Scaling Behaviors
//!
//! Elements can scale in different ways as zoom level (`k`) changes:
//!
//! - [`ScaleBehavior::World`]: Scales with zoom. Size in world units stays constant,
//!   so it appears larger when zoomed in, smaller when zoomed out.
//! - [`ScaleBehavior::Screen`]: Constant screen size. Divides by `k` to counteract
//!   the canvas transform, maintaining fixed pixel size.
//! - [`ScaleBehavior::Clamped`]: World-space scaling with min/max screen-size bounds.
//!   Useful for elements that should scale but not become too small or too large.

/// Defines how a visual property scales with zoom level.
#[derive(Clone, Debug)]
#[allow(
	dead_code,
	reason = "World/Screen variants complete the API for users customizing ScaleConfig"
)]
pub enum ScaleBehavior {
	/// Constant world-space size. Appears larger when zoomed in.
	World,
	/// Constant screen-space size (pixels). Unaffected by zoom.
	Screen,
	/// World-space scaling, clamped to min/max screen-space bounds.
	/// `(min_screen_px, max_screen_px)` - use `f64::NEG_INFINITY` or `f64::INFINITY` for unbounded.
	Clamped { min_screen: f64, max_screen: f64 },
}

impl ScaleBehavior {
	/// Compute the world-space value for a given base value and zoom level.
	///
	/// The returned value should be used directly in world-space drawing commands
	/// (after the canvas transform has been applied).
	pub fn apply(&self, base: f64, k: f64) -> f64 {
		match self {
			ScaleBehavior::World => base,
			ScaleBehavior::Screen => base / k,
			ScaleBehavior::Clamped {
				min_screen,
				max_screen,
			} => {
				// World-space base, but clamp the resulting screen size
				// screen_size = world_size * k
				// So: world_size = screen_size / k
				let min_world = min_screen / k;
				let max_world = max_screen / k;
				base.clamp(min_world, max_world)
			}
		}
	}
}

/// Defines how alpha/opacity scales with zoom level.
#[derive(Clone, Debug)]
#[allow(
	dead_code,
	reason = "Constant/Fade variants available for custom alpha behaviors"
)]
pub enum AlphaBehavior {
	/// Constant alpha regardless of zoom.
	Constant,
	/// Alpha scales linearly with zoom, clamped to [0, 1].
	/// At k=1, alpha = base. At k=0.5, alpha = base * 0.5.
	ScaleWithZoom,
	/// Alpha fades based on zoom thresholds.
	/// Fully visible at `full_alpha_k`, fades to zero at `zero_alpha_k`.
	Fade {
		zero_alpha_k: f64,
		full_alpha_k: f64,
	},
}

impl AlphaBehavior {
	/// Compute alpha multiplier for a given zoom level.
	pub fn apply(&self, k: f64) -> f64 {
		match self {
			AlphaBehavior::Constant => 1.0,
			AlphaBehavior::ScaleWithZoom => k.clamp(0.0, 1.0),
			AlphaBehavior::Fade {
				zero_alpha_k,
				full_alpha_k,
			} => {
				if zero_alpha_k == full_alpha_k {
					return 1.0;
				}
				let t = (k - zero_alpha_k) / (full_alpha_k - zero_alpha_k);
				t.clamp(0.0, 1.0)
			}
		}
	}
}

/// Configuration for node visual scaling.
#[derive(Clone, Debug)]
pub struct NodeScaleConfig {
	/// Base node radius in world units.
	pub radius: f64,
	/// How the node radius scales with zoom.
	pub radius_behavior: ScaleBehavior,
	/// Hit detection radius in world units.
	pub hit_radius: f64,
	/// How hit radius scales with zoom.
	pub hit_behavior: ScaleBehavior,
	/// Label font size in screen pixels.
	pub label_size: f64,
	/// Minimum zoom level for label font scaling.
	pub label_min_k: f64,
}

/// Configuration for edge visual scaling.
#[derive(Clone, Debug)]
pub struct EdgeScaleConfig {
	/// Base line width in screen pixels.
	pub line_width: f64,
	/// Dash pattern (dash, gap) in world units.
	pub dash_pattern: (f64, f64),
	/// Flow animation speed (world units per second).
	pub flow_speed: f64,
}

/// Configuration for arrow visual scaling.
#[derive(Clone, Debug)]
pub struct ArrowScaleConfig {
	/// Base arrow size in world units.
	pub size: f64,
	/// How arrow size scales with zoom.
	pub size_behavior: ScaleBehavior,
	/// How arrow alpha scales with zoom.
	pub alpha_behavior: AlphaBehavior,
	/// Minimum alpha to bother drawing.
	pub cull_alpha: f64,
}

/// Configuration for hover glow effects.
#[derive(Clone, Debug)]
pub struct GlowScaleConfig {
	/// Glow radius multiplier for hovered nodes.
	pub hovered_radius: f64,
	/// Glow radius multiplier for neighbor nodes.
	pub neighbor_radius: f64,
	/// Stroke width for hover ring in screen pixels.
	pub ring_width: f64,
	/// Ring offset from node edge in screen pixels.
	pub ring_offset: f64,
}

/// Complete scale configuration for all graph elements.
#[derive(Clone, Debug)]
pub struct ScaleConfig {
	pub node: NodeScaleConfig,
	pub edge: EdgeScaleConfig,
	pub arrow: ArrowScaleConfig,
	pub glow: GlowScaleConfig,
}

impl Default for ScaleConfig {
	fn default() -> Self {
		Self {
			node: NodeScaleConfig {
				radius: 5.0,
				radius_behavior: ScaleBehavior::Clamped {
					min_screen: 5.0,
					max_screen: f64::INFINITY,
				},
				hit_radius: 12.0,
				hit_behavior: ScaleBehavior::Clamped {
					min_screen: 5.0,
					max_screen: f64::INFINITY,
				},
				label_size: 10.0,
				label_min_k: 0.5,
			},
			edge: EdgeScaleConfig {
				line_width: 1.5,
				dash_pattern: (8.0, 4.0),
				flow_speed: 30.0,
			},
			arrow: ArrowScaleConfig {
				size: 5.0,
				size_behavior: ScaleBehavior::Clamped {
					min_screen: 0.0,
					max_screen: 18.0,
				},
				alpha_behavior: AlphaBehavior::ScaleWithZoom,
				cull_alpha: 0.05,
			},
			glow: GlowScaleConfig {
				hovered_radius: 3.0,
				neighbor_radius: 2.0,
				ring_width: 1.5,
				ring_offset: 2.0,
			},
		}
	}
}

/// Pre-computed scale values for a specific zoom level.
///
/// Create this once per frame and pass it to rendering functions.
/// All sizes are in world-space (ready to use after canvas transform).
#[derive(Clone, Debug)]
#[allow(
	dead_code,
	reason = "k field useful for debugging and future zoom-dependent logic"
)]
pub struct ScaledValues {
	/// Current zoom level.
	pub k: f64,
	/// Node radius in world-space.
	pub node_radius: f64,
	/// Hit detection radius in world-space.
	pub hit_radius: f64,
	/// Label font size string (e.g., "10px sans-serif").
	pub label_font: String,
	/// Edge line width in world-space.
	pub edge_line_width: f64,
	/// Dash pattern in world-space.
	pub dash_pattern: (f64, f64),
	/// Arrow size in world-space.
	pub arrow_size: f64,
	/// Arrow alpha multiplier [0, 1].
	pub arrow_alpha: f64,
	/// Whether to skip drawing arrows (alpha below threshold).
	pub cull_arrows: bool,
	/// Hover ring width in world-space.
	pub ring_width: f64,
	/// Hover ring offset in world-space.
	pub ring_offset: f64,
}

impl ScaledValues {
	/// Compute scaled values from configuration and current zoom level.
	pub fn new(config: &ScaleConfig, k: f64) -> Self {
		let node_radius = config.node.radius_behavior.apply(config.node.radius, k);
		let hit_radius = config.node.hit_behavior.apply(config.node.hit_radius, k);
		let label_font_size = config.node.label_size / k.max(config.node.label_min_k);
		let arrow_alpha = config.arrow.alpha_behavior.apply(k);

		Self {
			k,
			node_radius,
			hit_radius,
			label_font: format!("{}px sans-serif", label_font_size),
			edge_line_width: config.edge.line_width / k,
			dash_pattern: config.edge.dash_pattern,
			arrow_size: config.arrow.size_behavior.apply(config.arrow.size, k),
			arrow_alpha,
			cull_arrows: arrow_alpha < config.arrow.cull_alpha,
			ring_width: config.glow.ring_width / k,
			ring_offset: config.glow.ring_offset / k,
		}
	}

	/// Compute dash offset for flow animation.
	pub fn dash_offset(&self, flow_time: f64, flow_speed: f64) -> f64 {
		-flow_time * flow_speed
	}
}
