//! Client entrypoint for the CSR build.

// Bin target reuses lib deps, silence noisy lint.
#![allow(unused_crate_dependencies)]

use force_graph_canvas::{App, init_logging};
use leptos::prelude::*;

fn main() {
	init_logging();

	mount_to_body(|| {
		view! { <App /> }
	})
}
