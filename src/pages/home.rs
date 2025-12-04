use leptos::prelude::*;

use crate::components::force_graph::{ForceGraphCanvas, GraphData, GraphLink, GraphNode};

/// Generate sample graph data (random tree similar to the JS example).
fn generate_sample_data(n: usize) -> GraphData {
	let nodes: Vec<GraphNode> = (0..n)
		.map(|i| GraphNode {
			id: i.to_string(),
			label: if i < 10 {
				Some(format!("Node {}", i))
			} else {
				None
			},
			color: None,
			group: Some((i % 10) as u32),
		})
		.collect();

	let links: Vec<GraphLink> = (1..n)
		.map(|i| {
			let target = (rand_simple(i) * (i as f64)) as usize;
			GraphLink {
				source: i.to_string(),
				target: target.to_string(),
			}
		})
		.collect();

	GraphData { nodes, links }
}

/// Simple pseudo-random number generator (deterministic for consistency).
fn rand_simple(seed: usize) -> f64 {
	let x = ((seed + 1) * 9301 + 49297) % 233280;
	(x as f64) / 233280.0
}

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
	// Create graph data signal
	let graph_data = Signal::derive(move || generate_sample_data(100));

	view! {
		<ErrorBoundary fallback=|errors| {
			view! {
				<h1>"Uh oh! Something went wrong!"</h1>

				<p>"Errors: "</p>
				<ul>
					{move || {
						errors
							.get()
							.into_iter()
							.map(|(_, e)| view! { <li>{e.to_string()}</li> })
							.collect_view()
					}}
				</ul>
			}
		}>

			<div class="fullscreen-graph">
				<ForceGraphCanvas data=graph_data fullscreen=true />
				<div class="graph-overlay">
					<h1>"Force-Directed Graph"</h1>
					<p class="subtitle">"Drag nodes to reposition. Scroll to zoom. Drag background to pan."</p>
				</div>
			</div>
		</ErrorBoundary>
	}
}
