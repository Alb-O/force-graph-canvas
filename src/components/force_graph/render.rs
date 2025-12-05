//! Canvas rendering for the force graph.
//!
//! Handles all drawing operations: background, edges, nodes, labels, and effects.
//! Rendering uses multiple passes for correct z-ordering:
//! 1. Background and particles (screen space)
//! 2. Edge glows, then edge lines (world space)
//! 3. Node glows, non-highlighted nodes, then highlighted nodes on top

use std::f64::consts::PI;

use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use super::particles::ParticleSystem;
use super::scale::{ScaleConfig, ScaledValues};
use super::state::{ForceGraphState, NodeInfo};
use super::theme::{Color, Theme};

/// Attempt to smooth values that would otherwise cause abrupt visual changes.
fn smooth_step(t: f64) -> f64 {
	t * t * (3.0 - 2.0 * t)
}

/// Renders the complete graph to the canvas.
pub fn render(
	state: &ForceGraphState,
	ctx: &CanvasRenderingContext2d,
	config: &ScaleConfig,
	theme: &Theme,
	particles: Option<&ParticleSystem>,
) {
	let scale = ScaledValues::new(config, state.transform.k);

	draw_background(state, ctx, theme);

	if let Some(ps) = particles {
		draw_particles(state, ctx, theme, ps);
	}

	ctx.save();
	let _ = ctx.translate(state.transform.x, state.transform.y);
	let _ = ctx.scale(state.transform.k, state.transform.k);

	draw_edges(state, ctx, config, &scale, theme);
	draw_nodes(state, ctx, config, &scale, theme);

	ctx.restore();

	if theme.background.vignette > 0.0 {
		draw_vignette(state, ctx, theme);
	}
}

fn draw_background(state: &ForceGraphState, ctx: &CanvasRenderingContext2d, theme: &Theme) {
	if theme.background.use_gradient {
		let gradient = ctx
			.create_radial_gradient(
				state.width / 2.0,
				state.height / 2.0,
				0.0,
				state.width / 2.0,
				state.height / 2.0,
				(state.width.max(state.height)) * 0.8,
			)
			.unwrap();

		gradient
			.add_color_stop(0.0, &theme.background.color_secondary.to_css())
			.unwrap();
		gradient
			.add_color_stop(1.0, &theme.background.color.to_css())
			.unwrap();

		#[allow(deprecated)]
		ctx.set_fill_style(&gradient);
	} else {
		ctx.set_fill_style_str(&theme.background.color.to_css());
	}

	ctx.fill_rect(0.0, 0.0, state.width, state.height);
}

fn draw_vignette(state: &ForceGraphState, ctx: &CanvasRenderingContext2d, theme: &Theme) {
	let gradient = ctx
		.create_radial_gradient(
			state.width / 2.0,
			state.height / 2.0,
			state.width.min(state.height) * 0.3,
			state.width / 2.0,
			state.height / 2.0,
			state.width.max(state.height) * 0.7,
		)
		.unwrap();

	gradient.add_color_stop(0.0, "rgba(0, 0, 0, 0)").unwrap();
	gradient
		.add_color_stop(
			1.0,
			&format!("rgba(0, 0, 0, {})", theme.background.vignette),
		)
		.unwrap();

	#[allow(deprecated)]
	ctx.set_fill_style(&gradient);
	ctx.fill_rect(0.0, 0.0, state.width, state.height);
}

fn draw_particles(
	state: &ForceGraphState,
	ctx: &CanvasRenderingContext2d,
	theme: &Theme,
	particles: &ParticleSystem,
) {
	let color = &theme.particles.color;

	for p in &particles.particles {
		let alpha = particles.twinkle_alpha(p, state.flow_time);
		ctx.set_fill_style_str(&format!(
			"rgba({}, {}, {}, {})",
			color.r, color.g, color.b, alpha
		));

		ctx.begin_path();
		let _ = ctx.arc(p.x, p.y, p.size, 0.0, PI * 2.0);
		ctx.fill();
	}
}

fn draw_edges(
	state: &ForceGraphState,
	ctx: &CanvasRenderingContext2d,
	config: &ScaleConfig,
	scale: &ScaledValues,
	theme: &Theme,
) {
	let dash_offset = scale.dash_offset(state.flow_time, config.edge.flow_speed);
	let k = scale.k;

	if theme.edge.glow_intensity > 0.0 {
		state.graph.visit_edges(|n1, n2, _| {
			draw_edge_glow(state, ctx, scale, theme, n1, n2);
		});
	}

	state.graph.visit_edges(|n1, n2, _| {
		draw_edge_main(state, ctx, config, scale, theme, n1, n2, dash_offset, k);
	});

	let _ = ctx.set_line_dash(&js_sys::Array::new());
}

fn draw_edge_glow(
	state: &ForceGraphState,
	ctx: &CanvasRenderingContext2d,
	scale: &ScaledValues,
	theme: &Theme,
	n1: &force_graph::Node<NodeInfo>,
	n2: &force_graph::Node<NodeInfo>,
) {
	let (x1, y1, x2, y2) = (n1.x() as f64, n1.y() as f64, n2.x() as f64, n2.y() as f64);
	let (dx, dy) = (x2 - x1, y2 - y1);
	let dist = (dx * dx + dy * dy).sqrt();
	if dist < 0.001 {
		return;
	}

	let edge_t = state.highlight.edge_intensity(n1.index(), n2.index());
	let max_t = state.highlight.max_intensity();

	let glow_alpha = if edge_t > 0.01 {
		theme.edge.glow_intensity * (0.6 + 0.4 * smooth_step(edge_t))
	} else if max_t > 0.01 {
		theme.edge.glow_intensity * (0.6 - 0.4 * smooth_step(max_t))
	} else {
		theme.edge.glow_intensity * 0.6
	};

	if glow_alpha < 0.01 {
		return;
	}

	let glow_width = scale.edge_line_width * 4.0;
	let glow_color = &theme.edge.glow_color;

	ctx.set_stroke_style_str(&format!(
		"rgba({}, {}, {}, {})",
		glow_color.r,
		glow_color.g,
		glow_color.b,
		glow_alpha * glow_color.a
	));
	ctx.set_line_width(glow_width);
	let _ = ctx.set_line_dash(&js_sys::Array::new());

	let (ux, uy) = (dx / dist, dy / dist);

	if theme.edge.curved && dist > scale.node_radius * 4.0 {
		draw_curved_edge(
			ctx,
			x1,
			y1,
			x2,
			y2,
			ux,
			uy,
			scale.node_radius,
			theme.edge.curve_tension,
		);
	} else {
		ctx.begin_path();
		ctx.move_to(x1 + ux * scale.node_radius, y1 + uy * scale.node_radius);
		ctx.line_to(x2 - ux * scale.node_radius, y2 - uy * scale.node_radius);
		ctx.stroke();
	}
}

#[allow(clippy::too_many_arguments)]
fn draw_edge_main(
	state: &ForceGraphState,
	ctx: &CanvasRenderingContext2d,
	_config: &ScaleConfig,
	scale: &ScaledValues,
	theme: &Theme,
	n1: &force_graph::Node<NodeInfo>,
	n2: &force_graph::Node<NodeInfo>,
	dash_offset: f64,
	_k: f64,
) {
	let (x1, y1, x2, y2) = (n1.x() as f64, n1.y() as f64, n2.x() as f64, n2.y() as f64);
	let (dx, dy) = (x2 - x1, y2 - y1);
	let dist = (dx * dx + dy * dy).sqrt();
	if dist < 0.001 {
		return;
	}

	let edge_t = smooth_step(state.highlight.edge_intensity(n1.index(), n2.index()));
	let max_t = smooth_step(state.highlight.max_intensity());

	let (edge_alpha, base_arrow_alpha, base_width) = if edge_t > 0.01 {
		(
			0.7 + 0.3 * edge_t,
			0.9 + 0.1 * edge_t,
			scale.edge_line_width * (1.0 + 0.4 * edge_t),
		)
	} else if max_t > 0.01 {
		(
			0.7 - 0.5 * max_t,
			0.9 - 0.6 * max_t,
			scale.edge_line_width * (1.0 - 0.3 * max_t),
		)
	} else {
		(0.7, 0.9, scale.edge_line_width)
	};

	// Compensate for dash pattern fading to solid
	let width = base_width * (1.0 + 0.3 * (1.0 - scale.dash_alpha));
	let arrow_alpha = base_arrow_alpha * scale.arrow_alpha;

	let edge_color = &theme.edge.color;
	ctx.set_stroke_style_str(&format!(
		"rgba({}, {}, {}, {})",
		edge_color.r,
		edge_color.g,
		edge_color.b,
		edge_alpha * edge_color.a
	));
	ctx.set_line_width(width);

	// Fade dash pattern to solid when zoomed out
	let effective_gap = scale.dash_pattern.1 * scale.dash_alpha;
	if effective_gap > 0.1 {
		let _ = ctx.set_line_dash(&js_sys::Array::of2(
			&JsValue::from_f64(scale.dash_pattern.0),
			&JsValue::from_f64(effective_gap),
		));
		ctx.set_line_dash_offset(dash_offset);
	} else {
		let _ = ctx.set_line_dash(&js_sys::Array::new());
	}

	let (ux, uy) = (dx / dist, dy / dist);

	if theme.edge.curved && dist > scale.node_radius * 4.0 {
		draw_curved_edge(
			ctx,
			x1,
			y1,
			x2,
			y2,
			ux,
			uy,
			scale.node_radius + scale.arrow_size,
			theme.edge.curve_tension,
		);
	} else {
		ctx.begin_path();
		ctx.move_to(x1 + ux * scale.node_radius, y1 + uy * scale.node_radius);
		ctx.line_to(
			x2 - ux * (scale.node_radius + scale.arrow_size),
			y2 - uy * (scale.node_radius + scale.arrow_size),
		);
		ctx.stroke();
	}

	if !scale.cull_arrows && arrow_alpha > 0.0 {
		let _ = ctx.set_line_dash(&js_sys::Array::new());
		ctx.set_fill_style_str(&format!(
			"rgba({}, {}, {}, {})",
			edge_color.r,
			edge_color.g,
			edge_color.b,
			arrow_alpha * edge_color.a
		));

		let (tip_x, tip_y) = (x2 - ux * scale.node_radius, y2 - uy * scale.node_radius);
		let (back_x, back_y) = (tip_x - ux * scale.arrow_size, tip_y - uy * scale.arrow_size);
		let (px, py) = (-uy * scale.arrow_size * 0.5, ux * scale.arrow_size * 0.5);

		ctx.begin_path();
		ctx.move_to(tip_x, tip_y);
		ctx.line_to(back_x + px, back_y + py);
		ctx.line_to(back_x - px, back_y - py);
		ctx.close_path();
		ctx.fill();
	}
}

#[allow(clippy::too_many_arguments)]
fn draw_curved_edge(
	ctx: &CanvasRenderingContext2d,
	x1: f64,
	y1: f64,
	x2: f64,
	y2: f64,
	ux: f64,
	uy: f64,
	offset: f64,
	tension: f64,
) {
	let (dx, dy) = (x2 - x1, y2 - y1);
	let dist = (dx * dx + dy * dy).sqrt();

	let curve_offset = dist * tension * 0.3;
	let (px, py) = (-uy * curve_offset, ux * curve_offset);

	let (start_x, start_y) = (x1 + ux * offset, y1 + uy * offset);
	let (end_x, end_y) = (x2 - ux * offset, y2 - uy * offset);
	let (mid_x, mid_y) = ((x1 + x2) / 2.0 + px, (y1 + y2) / 2.0 + py);

	ctx.begin_path();
	ctx.move_to(start_x, start_y);
	let _ = ctx.quadratic_curve_to(mid_x, mid_y, end_x, end_y);
	ctx.stroke();
}

fn draw_nodes(
	state: &ForceGraphState,
	ctx: &CanvasRenderingContext2d,
	_config: &ScaleConfig,
	scale: &ScaledValues,
	theme: &Theme,
) {
	let max_t = smooth_step(state.highlight.max_intensity());
	let has_highlight = max_t > 0.01;
	let pulse = if theme.node.pulse_intensity > 0.0 {
		(state.flow_time * theme.node.pulse_speed).sin() * theme.node.pulse_intensity
	} else {
		0.0
	};

	// Pass 1: node glows
	if theme.node.glow_intensity > 0.0 {
		state.graph.visit_nodes(|node| {
			let idx = node.index();
			let node_t = smooth_step(state.highlight.node_intensity(idx));
			let hover_t = smooth_step(state.highlight.hover_ring_intensity(idx));

			let glow_mult = if node_t > 0.001 {
				let neighbor_glow = 1.0 + 0.3 * node_t;
				let hovered_glow = 1.5 + 0.5 * node_t;
				neighbor_glow + (hovered_glow - neighbor_glow) * hover_t
			} else if has_highlight {
				1.0 - 0.7 * max_t
			} else {
				1.0
			};

			draw_node_glow(ctx, node, scale, theme, glow_mult, pulse);
		});
	}

	// Pass 2: non-highlighted nodes
	state.graph.visit_nodes(|node| {
		let idx = node.index();
		let node_t = state.highlight.node_intensity(idx);
		if node_t > 0.001 {
			return;
		}
		let (alpha, radius_mult) = if has_highlight {
			(1.0 - 0.7 * max_t, 1.0 - 0.15 * max_t)
		} else {
			(1.0, 1.0)
		};
		draw_node(ctx, node, scale, theme, alpha, radius_mult, pulse);
	});

	// Pass 3: highlighted/transitioning nodes on top
	state.graph.visit_nodes(|node| {
		let idx = node.index();
		let node_t = state.highlight.node_intensity(idx);
		if node_t <= 0.001 {
			return;
		}

		let eased_t = smooth_step(node_t);
		let hover_t = smooth_step(state.highlight.hover_ring_intensity(idx));
		let (x, y) = (node.x() as f64, node.y() as f64);

		let dim_alpha = if has_highlight {
			1.0 - 0.7 * max_t
		} else {
			1.0
		};
		let dim_radius = if has_highlight {
			1.0 - 0.15 * max_t
		} else {
			1.0
		};

		let neighbor_radius = 1.0 + 0.25 * eased_t;
		let hovered_radius = 1.0 + 0.4 * eased_t;
		let highlight_radius = neighbor_radius + (hovered_radius - neighbor_radius) * hover_t;

		let alpha = dim_alpha + (1.0 - dim_alpha) * eased_t;
		let radius_mult = dim_radius + (highlight_radius - dim_radius) * eased_t;

		draw_node(ctx, node, scale, theme, alpha, radius_mult, pulse);

		let ring_t = smooth_step(state.highlight.hover_ring_intensity(idx));
		if ring_t > 0.01 {
			let node_size = node.data.user_data.size;
			let radius = scale.node_radius * radius_mult * node_size * (1.0 + pulse);
			ctx.begin_path();
			let _ = ctx.arc(x, y, radius + scale.ring_offset, 0.0, 2.0 * PI);
			ctx.set_stroke_style_str(&format!("rgba(255, 255, 255, {})", 0.8 * ring_t));
			ctx.set_line_width(scale.ring_width);
			ctx.stroke();

			ctx.begin_path();
			let _ = ctx.arc(x, y, radius + scale.ring_offset * 2.5, 0.0, 2.0 * PI);
			ctx.set_stroke_style_str(&format!("rgba(255, 255, 255, {})", 0.3 * ring_t));
			ctx.set_line_width(scale.ring_width * 0.5);
			ctx.stroke();
		}

		if let Some(label) = &node.data.user_data.label {
			let node_size = node.data.user_data.size;
			let radius = scale.node_radius * radius_mult * node_size * (1.0 + pulse);
			ctx.set_fill_style_str(&format!("rgba(255, 255, 255, {})", 0.95 * alpha));
			ctx.set_font(&scale.label_font);
			let _ = ctx.fill_text(label, x + radius + 4.0, y + 3.0);
		}
	});
}

fn draw_node_glow(
	ctx: &CanvasRenderingContext2d,
	node: &force_graph::Node<NodeInfo>,
	scale: &ScaledValues,
	theme: &Theme,
	intensity_mult: f64,
	pulse: f64,
) {
	let (x, y) = (node.x() as f64, node.y() as f64);
	let node_size = node.data.user_data.size;
	let radius = scale.node_radius * node_size * (1.0 + pulse);
	let glow_radius = radius * 3.0 * intensity_mult;
	let alpha = theme.node.glow_intensity * intensity_mult * 0.4;

	if alpha < 0.01 {
		return;
	}

	let node_color = parse_color(&node.data.user_data.color);

	let gradient = ctx
		.create_radial_gradient(x, y, radius * 0.5, x, y, glow_radius)
		.unwrap();

	let glow_color = node_color.with_alpha(alpha * theme.node.glow_saturation);
	let white_glow = Color::rgba(255, 255, 255, alpha * 0.3);

	gradient
		.add_color_stop(0.0, &white_glow.lerp(glow_color, 0.5).to_css())
		.unwrap();
	gradient
		.add_color_stop(0.4, &glow_color.with_alpha(alpha * 0.5).to_css())
		.unwrap();
	gradient.add_color_stop(1.0, "rgba(0, 0, 0, 0)").unwrap();

	ctx.begin_path();
	let _ = ctx.arc(x, y, glow_radius, 0.0, 2.0 * PI);
	#[allow(deprecated)]
	ctx.set_fill_style(&gradient);
	ctx.fill();
}

fn draw_node(
	ctx: &CanvasRenderingContext2d,
	node: &force_graph::Node<NodeInfo>,
	scale: &ScaledValues,
	theme: &Theme,
	alpha: f64,
	radius_mult: f64,
	pulse: f64,
) {
	let (x, y) = (node.x() as f64, node.y() as f64);
	let node_size = node.data.user_data.size;
	let radius = scale.node_radius * radius_mult * node_size * (1.0 + pulse);
	let color = &node.data.user_data.color;

	ctx.set_global_alpha(alpha);

	if theme.node.use_gradient {
		let gradient = ctx
			.create_radial_gradient(x - radius * 0.3, y - radius * 0.3, 0.0, x, y, radius)
			.unwrap();

		let base_color = parse_color(color);
		let highlight = base_color.lighten(0.4);
		let shadow = base_color.darken(0.2);

		gradient.add_color_stop(0.0, &highlight.to_css()).unwrap();
		gradient.add_color_stop(0.7, &base_color.to_css()).unwrap();
		gradient.add_color_stop(1.0, &shadow.to_css()).unwrap();

		ctx.begin_path();
		let _ = ctx.arc(x, y, radius, 0.0, 2.0 * PI);
		#[allow(deprecated)]
		ctx.set_fill_style(&gradient);
		ctx.fill();
	} else {
		ctx.begin_path();
		let _ = ctx.arc(x, y, radius, 0.0, 2.0 * PI);
		ctx.set_fill_style_str(color);
		ctx.fill();
	}

	if theme.node.border_width > 0.0 {
		ctx.begin_path();
		let _ = ctx.arc(x, y, radius, 0.0, 2.0 * PI);
		ctx.set_stroke_style_str(&theme.node.border_color.to_css());
		ctx.set_line_width(theme.node.border_width / scale.k);
		ctx.stroke();
	}

	ctx.set_global_alpha(1.0);

	if let Some(label) = &node.data.user_data.label {
		if alpha > 0.5 {
			ctx.set_global_alpha(alpha * 0.8);
			ctx.set_fill_style_str("rgba(255, 255, 255, 0.85)");
			ctx.set_font(&scale.label_font);
			let _ = ctx.fill_text(label, x + radius + 4.0, y + 3.0);
			ctx.set_global_alpha(1.0);
		}
	}
}

/// Parses a CSS color string into a [`Color`].
/// Supports hex (`#RRGGBB`) and `rgb()`/`rgba()` functional notation.
fn parse_color(color_str: &str) -> Color {
	if color_str.starts_with('#') && color_str.len() == 7 {
		let r = u8::from_str_radix(&color_str[1..3], 16).unwrap_or(128);
		let g = u8::from_str_radix(&color_str[3..5], 16).unwrap_or(128);
		let b = u8::from_str_radix(&color_str[5..7], 16).unwrap_or(128);
		Color::rgb(r, g, b)
	} else if color_str.starts_with("rgb") {
		let nums: Vec<&str> = color_str
			.trim_start_matches("rgba(")
			.trim_start_matches("rgb(")
			.trim_end_matches(')')
			.split(',')
			.collect();
		let r = nums
			.first()
			.and_then(|s| s.trim().parse().ok())
			.unwrap_or(128);
		let g = nums
			.get(1)
			.and_then(|s| s.trim().parse().ok())
			.unwrap_or(128);
		let b = nums
			.get(2)
			.and_then(|s| s.trim().parse().ok())
			.unwrap_or(128);
		let a = nums
			.get(3)
			.and_then(|s| s.trim().parse().ok())
			.unwrap_or(1.0);
		Color::rgba(r, g, b, a)
	} else {
		Color::rgb(128, 128, 128)
	}
}
