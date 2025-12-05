use std::f64::consts::PI;

use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use super::state::{ForceGraphState, MIN_SCREEN_RADIUS, NODE_RADIUS};

/// Returns the effective node radius in world-space, ensuring a minimum screen-space size.
/// When zoomed out (k < 1), nodes would normally shrink; this clamps them to remain visible.
fn effective_radius(base_radius: f64, k: f64) -> f64 {
	// min_world_radius is how big the radius needs to be in world-space
	// to appear as MIN_SCREEN_RADIUS on screen
	let min_world_radius = MIN_SCREEN_RADIUS / k;
	base_radius.max(min_world_radius)
}

fn ease_out_cubic(t: f64) -> f64 {
	1.0 - (1.0 - t).powi(3)
}

pub fn render(state: &ForceGraphState, ctx: &CanvasRenderingContext2d) {
	ctx.set_fill_style_str("#1a1a2e");
	ctx.fill_rect(0.0, 0.0, state.width, state.height);
	ctx.save();
	let _ = ctx.translate(state.transform.x, state.transform.y);
	let _ = ctx.scale(state.transform.k, state.transform.k);
	draw_edges(state, ctx);
	draw_nodes(state, ctx);
	ctx.restore();
}

fn draw_edges(state: &ForceGraphState, ctx: &CanvasRenderingContext2d) {
	let k = state.transform.k;
	// Line width scales inversely with zoom for constant screen size.
	// Dash pattern stays in world-space and scales with content.
	let line_width = 1.5 / k;
	// Arrows: fade out when zoomed out, prominent when zoomed in.
	// At k=1: size=5, alpha=1. At k=0.5: size=2.5, alpha=0.5. At k>3.6: clamped to 18px screen.
	let max_arrow_screen_size = 18.0;
	let arrow_size = (5.0_f64).min(max_arrow_screen_size / k); // world-space, clamped to max screen size
	let arrow_alpha_scale = k.clamp(0.0, 1.0); // fade out when zoomed out
	let (dash, gap) = (8.0, 4.0);
	let dash_offset = -state.flow_time * 30.0;
	let t = ease_out_cubic(state.hover.highlight_t);
	let node_radius = effective_radius(NODE_RADIUS, k);

	state.graph.visit_edges(|n1, n2, _| {
		let (x1, y1, x2, y2) = (n1.x() as f64, n1.y() as f64, n2.x() as f64, n2.y() as f64);
		let (dx, dy) = (x2 - x1, y2 - y1);
		let dist = (dx * dx + dy * dy).sqrt();
		if dist < 0.001 {
			return;
		}

		let is_highlighted = state.is_highlighted(n1.index()) && state.is_highlighted(n2.index());

		// Base values when no highlight active
		// When highlighting: highlighted edges brighten, others dim
		// t=0: all edges at base (0.6), t=1: highlighted at 0.9, others at 0.15
		let (edge_alpha, base_arrow_alpha, width) = if is_highlighted {
			(0.6 + 0.3 * t, 0.8 + 0.1 * t, line_width * (1.0 + 0.3 * t))
		} else {
			(0.6 - 0.45 * t, 0.8 - 0.45 * t, line_width * (1.0 - 0.3 * t))
		};
		let arrow_alpha = base_arrow_alpha * arrow_alpha_scale;

		ctx.set_stroke_style_str(&format!("rgba(100, 180, 255, {})", edge_alpha));
		ctx.set_line_width(width);
		let _ = ctx.set_line_dash(&js_sys::Array::of2(
			&JsValue::from_f64(dash),
			&JsValue::from_f64(gap),
		));
		ctx.set_line_dash_offset(dash_offset);

		let (ux, uy) = (dx / dist, dy / dist);
		ctx.begin_path();
		ctx.move_to(x1 + ux * node_radius, y1 + uy * node_radius);
		ctx.line_to(
			x2 - ux * (node_radius + arrow_size),
			y2 - uy * (node_radius + arrow_size),
		);
		ctx.stroke();

		// Skip drawing arrows if they'd be nearly invisible
		if arrow_alpha > 0.05 {
			let _ = ctx.set_line_dash(&js_sys::Array::new());
			ctx.set_fill_style_str(&format!("rgba(100, 180, 255, {})", arrow_alpha));
			let (tip_x, tip_y) = (x2 - ux * node_radius, y2 - uy * node_radius);
			let (back_x, back_y) = (tip_x - ux * arrow_size, tip_y - uy * arrow_size);
			let (px, py) = (-uy * arrow_size * 0.5, ux * arrow_size * 0.5);
			ctx.begin_path();
			ctx.move_to(tip_x, tip_y);
			ctx.line_to(back_x + px, back_y + py);
			ctx.line_to(back_x - px, back_y - py);
			ctx.close_path();
			ctx.fill();
		}
	});
	let _ = ctx.set_line_dash(&js_sys::Array::new());
}

fn draw_nodes(state: &ForceGraphState, ctx: &CanvasRenderingContext2d) {
	let (has_highlight, t, k) = (
		state.has_active_highlight(),
		ease_out_cubic(state.hover.highlight_t),
		state.transform.k,
	);
	let base_radius = effective_radius(NODE_RADIUS, k);

	state.graph.visit_nodes(|node| {
		let idx = node.index();
		if has_highlight && state.is_highlighted(idx) {
			return;
		}
		let (x, y) = (node.x() as f64, node.y() as f64);
		let (alpha, radius) = (1.0 - 0.7 * t, base_radius * (1.0 - 0.15 * t));

		ctx.set_global_alpha(alpha);
		ctx.begin_path();
		let _ = ctx.arc(x, y, radius, 0.0, 2.0 * PI);
		ctx.set_fill_style_str(&node.data.user_data.color);
		ctx.fill();
		ctx.set_global_alpha(1.0);

		if let Some(label) = &node.data.user_data.label {
			ctx.set_fill_style_str(&format!("rgba(255, 255, 255, {})", alpha * 0.8));
			ctx.set_font(&format!("{}px sans-serif", 10.0 / k.max(0.5)));
			let _ = ctx.fill_text(label, x + radius + 3.0, y + 3.0);
		}
	});

	if !has_highlight {
		return;
	}

	state.graph.visit_nodes(|node| {
		let idx = node.index();
		if !state.is_highlighted(idx) {
			return;
		}
		let (x, y) = (node.x() as f64, node.y() as f64);
		let is_hovered = state.is_hovered(idx);
		let is_neighbor =
			state.hover.neighbors.contains(&idx) || state.hover.prev_neighbors.contains(&idx);

		let (radius, glow_radius) = if is_hovered {
			(
				base_radius * (1.0 + 0.35 * t),
				base_radius * (1.8 + 1.2 * t),
			)
		} else if is_neighbor {
			(base_radius * (1.0 + 0.2 * t), base_radius * (1.4 + 0.6 * t))
		} else {
			(base_radius, 0.0)
		};

		if glow_radius > 0.0 && t > 0.01 {
			let gradient = ctx
				.create_radial_gradient(x, y, radius * 0.3, x, y, glow_radius)
				.unwrap();
			let alpha = if is_hovered { 0.35 * t } else { 0.2 * t };
			gradient
				.add_color_stop(0.0, &format!("rgba(255, 255, 255, {})", alpha))
				.unwrap();
			gradient
				.add_color_stop(0.6, &format!("rgba(200, 220, 255, {})", alpha * 0.3))
				.unwrap();
			gradient
				.add_color_stop(1.0, "rgba(255, 255, 255, 0)")
				.unwrap();
			ctx.begin_path();
			let _ = ctx.arc(x, y, glow_radius, 0.0, 2.0 * PI);
			#[allow(deprecated)]
			ctx.set_fill_style(&gradient);
			ctx.fill();
		}

		ctx.begin_path();
		let _ = ctx.arc(x, y, radius, 0.0, 2.0 * PI);
		ctx.set_fill_style_str(&node.data.user_data.color);
		ctx.fill();

		if is_hovered && t > 0.01 {
			ctx.begin_path();
			let _ = ctx.arc(x, y, radius + 2.0 / k, 0.0, 2.0 * PI);
			ctx.set_stroke_style_str(&format!("rgba(255, 255, 255, {})", 0.7 * t));
			ctx.set_line_width(1.5 / k);
			ctx.stroke();
		}

		if let Some(label) = &node.data.user_data.label {
			ctx.set_fill_style_str("white");
			ctx.set_font(&format!("{}px sans-serif", 10.0 / k.max(0.5)));
			let _ = ctx.fill_text(label, x + radius + 3.0, y + 3.0);
		}
	});
}
