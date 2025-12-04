use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, WheelEvent, Window};

use super::render;
use super::state::ForceGraphState;
use super::types::GraphData;

#[component]
pub fn ForceGraphCanvas(
	#[prop(into)] data: Signal<GraphData>,
	#[prop(default = false)] fullscreen: bool,
	#[prop(default = None)] width: Option<f64>,
	#[prop(default = None)] height: Option<f64>,
) -> impl IntoView {
	let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
	let state: Rc<RefCell<Option<ForceGraphState>>> = Rc::new(RefCell::new(None));
	let animate: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
	let resize_cb: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
	let (state_init, animate_init, resize_cb_init) =
		(state.clone(), animate.clone(), resize_cb.clone());

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
		*state_init.borrow_mut() = Some(ForceGraphState::new(&data.get(), w, h));

		if fullscreen {
			let (state_resize, canvas_resize) = (state_init.clone(), canvas.clone());
			*resize_cb_init.borrow_mut() = Some(Closure::new(move || {
				let win: Window = web_sys::window().unwrap();
				let (nw, nh) = (
					win.inner_width().unwrap().as_f64().unwrap(),
					win.inner_height().unwrap().as_f64().unwrap(),
				);
				canvas_resize.set_width(nw as u32);
				canvas_resize.set_height(nh as u32);
				if let Some(ref mut s) = *state_resize.borrow_mut() {
					s.resize(nw, nh);
				}
			}));
			if let Some(ref cb) = *resize_cb_init.borrow() {
				let _ =
					window.add_event_listener_with_callback("resize", cb.as_ref().unchecked_ref());
			}
		}

		let (state_anim, animate_inner) = (state_init.clone(), animate_init.clone());
		*animate_init.borrow_mut() = Some(Closure::new(move || {
			if let Some(ref mut s) = *state_anim.borrow_mut() {
				if s.animation_running {
					s.tick(0.016);
				}
				render::render(s, &ctx);
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

	let state_md = state.clone();
	let on_mousedown = move |ev: MouseEvent| {
		let canvas: HtmlCanvasElement = canvas_ref.get().unwrap().into();
		let rect = canvas.get_bounding_client_rect();
		let (x, y) = (
			ev.client_x() as f64 - rect.left(),
			ev.client_y() as f64 - rect.top(),
		);

		if let Some(ref mut s) = *state_md.borrow_mut() {
			if let Some(idx) = s.node_at_position(x, y) {
				s.drag.active = true;
				s.drag.node_idx = Some(idx);
				s.drag.start_x = x;
				s.drag.start_y = y;
				s.graph.visit_nodes(|node| {
					if node.index() == idx {
						s.drag.node_start_x = node.x();
						s.drag.node_start_y = node.y();
					}
				});
			} else {
				s.pan.active = true;
				s.pan.start_x = x;
				s.pan.start_y = y;
				s.pan.transform_start_x = s.transform.x;
				s.pan.transform_start_y = s.transform.y;
			}
		}
	};

	let state_mm = state.clone();
	let on_mousemove = move |ev: MouseEvent| {
		let canvas: HtmlCanvasElement = canvas_ref.get().unwrap().into();
		let rect = canvas.get_bounding_client_rect();
		let (x, y) = (
			ev.client_x() as f64 - rect.left(),
			ev.client_y() as f64 - rect.top(),
		);

		if let Some(ref mut s) = *state_mm.borrow_mut() {
			// Update hover state when not dragging
			if !s.drag.active {
				let hovered = s.node_at_position(x, y);
				s.set_hover(hovered);
			}

			if s.drag.active {
				if let Some(idx) = s.drag.node_idx {
					let (dx, dy) = (
						(x - s.drag.start_x) / s.transform.k,
						(y - s.drag.start_y) / s.transform.k,
					);
					let (nx, ny) = (
						s.drag.node_start_x + dx as f32,
						s.drag.node_start_y + dy as f32,
					);
					s.graph.visit_nodes_mut(|node| {
						if node.index() == idx {
							node.data.x = nx;
							node.data.y = ny;
							node.data.is_anchor = true;
						}
					});
				}
			} else if s.pan.active {
				s.transform.x = s.pan.transform_start_x + (x - s.pan.start_x);
				s.transform.y = s.pan.transform_start_y + (y - s.pan.start_y);
			}
		}
	};

	let state_mu = state.clone();
	let on_mouseup = move |_: MouseEvent| {
		if let Some(ref mut s) = *state_mu.borrow_mut() {
			if s.drag.active {
				if let Some(idx) = s.drag.node_idx {
					s.graph.visit_nodes_mut(|node| {
						if node.index() == idx {
							node.data.is_anchor = true;
						}
					});
				}
			}
			s.drag.active = false;
			s.drag.node_idx = None;
			s.pan.active = false;
		}
	};

	let state_ml = state.clone();
	let on_mouseleave = move |_: MouseEvent| {
		if let Some(ref mut s) = *state_ml.borrow_mut() {
			s.drag.active = false;
			s.drag.node_idx = None;
			s.pan.active = false;
			s.set_hover(None);
		}
	};

	let state_wh = state.clone();
	let on_wheel = move |ev: WheelEvent| {
		ev.prevent_default();
		let canvas: HtmlCanvasElement = canvas_ref.get().unwrap().into();
		let rect = canvas.get_bounding_client_rect();
		let (x, y) = (
			ev.client_x() as f64 - rect.left(),
			ev.client_y() as f64 - rect.top(),
		);

		if let Some(ref mut s) = *state_wh.borrow_mut() {
			let factor = if ev.delta_y() > 0.0 { 0.9 } else { 1.1 };
			let new_k = (s.transform.k * factor).clamp(0.1, 10.0);
			let ratio = new_k / s.transform.k;
			s.transform.x = x - (x - s.transform.x) * ratio;
			s.transform.y = y - (y - s.transform.y) * ratio;
			s.transform.k = new_k;
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
