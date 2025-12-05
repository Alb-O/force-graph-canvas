//! Leptos component wrapping the force-directed graph canvas.
//!
//! The component creates an HTML canvas element and wires up mouse/wheel event
//! handlers for node dragging, panning, and zooming. An animation loop runs via
//! `requestAnimationFrame`, calling the physics simulation and renderer each frame.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, WheelEvent, Window};

use super::particles::ParticleSystem;
use super::render;
use super::scale::ScaleConfig;
use super::state::ForceGraphState;
use super::theme::Theme;
use super::types::GraphData;

/// Bundles graph simulation state with visual configuration (scaling, theme, particles).
struct GraphContext {
	state: ForceGraphState,
	scale: ScaleConfig,
	theme: Theme,
	particles: Option<ParticleSystem>,
}

/// Renders an interactive force-directed graph on a canvas element.
///
/// Pass graph data via the reactive `data` signal. The component sizes itself
/// to its parent container by default; set `fullscreen = true` to fill the
/// viewport and resize automatically with the window. Explicit `width`/`height`
/// override automatic sizing.
#[component]
pub fn ForceGraphCanvas(
	#[prop(into)] data: Signal<GraphData>,
	#[prop(default = false)] fullscreen: bool,
	#[prop(default = None)] width: Option<f64>,
	#[prop(default = None)] height: Option<f64>,
) -> impl IntoView {
	let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
	let context: Rc<RefCell<Option<GraphContext>>> = Rc::new(RefCell::new(None));
	let animate: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
	let resize_cb: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
	let (context_init, animate_init, resize_cb_init) =
		(context.clone(), animate.clone(), resize_cb.clone());

	Effect::new(move |_| {
		let Some(canvas) = canvas_ref.get() else {
			return;
		};
		let canvas: HtmlCanvasElement = canvas.into();
		let window: Window = web_sys::window().unwrap();

		let (w, h) = if fullscreen {
			(
				window.inner_width().unwrap().as_f64().unwrap(),
				window.inner_height().unwrap().as_f64().unwrap(),
			)
		} else {
			(
				width.unwrap_or_else(|| {
					canvas
						.parent_element()
						.map(|p| p.client_width() as f64)
						.unwrap_or(800.0)
				}),
				height.unwrap_or_else(|| {
					canvas
						.parent_element()
						.map(|p| p.client_height() as f64)
						.unwrap_or(600.0)
				}),
			)
		};
		canvas.set_width(w as u32);
		canvas.set_height(h as u32);

		let ctx: CanvasRenderingContext2d = canvas
			.get_context("2d")
			.unwrap()
			.unwrap()
			.dyn_into()
			.unwrap();

		let theme = Theme::default();
		let particles = if theme.particles.enabled {
			Some(ParticleSystem::new(&theme.particles, w, h))
		} else {
			None
		};

		*context_init.borrow_mut() = Some(GraphContext {
			state: ForceGraphState::new(&data.get(), w, h, &theme),
			scale: ScaleConfig::default(),
			theme,
			particles,
		});

		if fullscreen {
			let (context_resize, canvas_resize) = (context_init.clone(), canvas.clone());
			*resize_cb_init.borrow_mut() = Some(Closure::new(move || {
				let win: Window = web_sys::window().unwrap();
				let (nw, nh) = (
					win.inner_width().unwrap().as_f64().unwrap(),
					win.inner_height().unwrap().as_f64().unwrap(),
				);
				canvas_resize.set_width(nw as u32);
				canvas_resize.set_height(nh as u32);
				if let Some(ref mut c) = *context_resize.borrow_mut() {
					c.state.resize(nw, nh);
					if let Some(ref mut ps) = c.particles {
						ps.resize(nw, nh);
					}
				}
			}));
			if let Some(ref cb) = *resize_cb_init.borrow() {
				let _ =
					window.add_event_listener_with_callback("resize", cb.as_ref().unchecked_ref());
			}
		}

		let (context_anim, animate_inner) = (context_init.clone(), animate_init.clone());
		*animate_init.borrow_mut() = Some(Closure::new(move || {
			if let Some(ref mut c) = *context_anim.borrow_mut() {
				let dt = 0.016;
				if c.state.animation_running {
					c.state.tick(dt as f32);
				}
				if let Some(ref mut ps) = c.particles {
					ps.update(dt);
				}
				render::render(&c.state, &ctx, &c.scale, &c.theme, c.particles.as_ref());
			}
			if let Some(ref cb) = *animate_inner.borrow() {
				let _ = web_sys::window()
					.unwrap()
					.request_animation_frame(cb.as_ref().unchecked_ref());
			}
		}));
		if let Some(ref cb) = *animate_init.borrow() {
			let _ = window.request_animation_frame(cb.as_ref().unchecked_ref());
		}
	});

	let context_md = context.clone();
	let on_mousedown = move |ev: MouseEvent| {
		let canvas: HtmlCanvasElement = canvas_ref.get().unwrap().into();
		let rect = canvas.get_bounding_client_rect();
		let (x, y) = (
			ev.client_x() as f64 - rect.left(),
			ev.client_y() as f64 - rect.top(),
		);

		if let Some(ref mut c) = *context_md.borrow_mut() {
			if let Some(idx) = c.state.node_at_position(x, y, &c.scale) {
				c.state.drag.active = true;
				c.state.drag.node_idx = Some(idx);
				c.state.drag.start_x = x;
				c.state.drag.start_y = y;
				c.state.graph.visit_nodes(|node| {
					if node.index() == idx {
						c.state.drag.node_start_x = node.x();
						c.state.drag.node_start_y = node.y();
					}
				});
			} else {
				c.state.pan.active = true;
				c.state.pan.start_x = x;
				c.state.pan.start_y = y;
				c.state.pan.transform_start_x = c.state.transform.x;
				c.state.pan.transform_start_y = c.state.transform.y;
			}
		}
	};

	let context_mm = context.clone();
	let on_mousemove = move |ev: MouseEvent| {
		let canvas: HtmlCanvasElement = canvas_ref.get().unwrap().into();
		let rect = canvas.get_bounding_client_rect();
		let (x, y) = (
			ev.client_x() as f64 - rect.left(),
			ev.client_y() as f64 - rect.top(),
		);

		if let Some(ref mut c) = *context_mm.borrow_mut() {
			// Update hover state when not dragging
			if !c.state.drag.active {
				let hovered = c.state.node_at_position(x, y, &c.scale);
				c.state.set_hover(hovered);
			}

			if c.state.drag.active {
				if let Some(idx) = c.state.drag.node_idx {
					let (dx, dy) = (
						(x - c.state.drag.start_x) / c.state.transform.k,
						(y - c.state.drag.start_y) / c.state.transform.k,
					);
					let (nx, ny) = (
						c.state.drag.node_start_x + dx as f32,
						c.state.drag.node_start_y + dy as f32,
					);
					c.state.graph.visit_nodes_mut(|node| {
						if node.index() == idx {
							node.data.x = nx;
							node.data.y = ny;
							node.data.is_anchor = true;
						}
					});
				}
			} else if c.state.pan.active {
				c.state.transform.x = c.state.pan.transform_start_x + (x - c.state.pan.start_x);
				c.state.transform.y = c.state.pan.transform_start_y + (y - c.state.pan.start_y);
			}
		}
	};

	let context_mu = context.clone();
	let on_mouseup = move |_: MouseEvent| {
		if let Some(ref mut c) = *context_mu.borrow_mut() {
			if c.state.drag.active {
				if let Some(idx) = c.state.drag.node_idx {
					c.state.graph.visit_nodes_mut(|node| {
						if node.index() == idx {
							node.data.is_anchor = true;
						}
					});
				}
			}
			c.state.drag.active = false;
			c.state.drag.node_idx = None;
			c.state.pan.active = false;
		}
	};

	let context_ml = context.clone();
	let on_mouseleave = move |_: MouseEvent| {
		if let Some(ref mut c) = *context_ml.borrow_mut() {
			c.state.drag.active = false;
			c.state.drag.node_idx = None;
			c.state.pan.active = false;
			c.state.set_hover(None);
		}
	};

	let context_wh = context.clone();
	let on_wheel = move |ev: WheelEvent| {
		ev.prevent_default();
		let canvas: HtmlCanvasElement = canvas_ref.get().unwrap().into();
		let rect = canvas.get_bounding_client_rect();
		let (x, y) = (
			ev.client_x() as f64 - rect.left(),
			ev.client_y() as f64 - rect.top(),
		);

		if let Some(ref mut c) = *context_wh.borrow_mut() {
			let factor = if ev.delta_y() > 0.0 { 0.9 } else { 1.1 };
			let new_k = (c.state.transform.k * factor).clamp(0.1, 10.0);
			let ratio = new_k / c.state.transform.k;
			c.state.transform.x = x - (x - c.state.transform.x) * ratio;
			c.state.transform.y = y - (y - c.state.transform.y) * ratio;
			c.state.transform.k = new_k;
		}
	};

	view! {
		<canvas
			node_ref=canvas_ref
			class="force-graph-canvas"
			on:mousedown=on_mousedown
			on:mousemove=on_mousemove
			on:mouseup=on_mouseup
			on:mouseleave=on_mouseleave
			on:wheel=on_wheel
			style="display: block; cursor: grab;"
		/>
	}
}
