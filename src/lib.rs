//! Leptos client-side app wiring and routes.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;
use log::{Level, info};

// Modules
mod components;
mod pages;

// Top-Level pages
use crate::pages::home::Home;
use crate::pages::not_found::NotFound;

/// Initialize logging and panic hooks for the WASM target.
pub fn init_logging() {
	let _ = console_log::init_with_level(Level::Debug);
	console_error_panic_hook::set_once();
	info!("Logging initialized");
}

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
	// Provides context that manages stylesheets, titles, meta tags, etc.
	provide_meta_context();

	view! {
		<Html attr:lang="en" attr:dir="ltr" attr:data-theme="light" />

		// sets the document title
		<Title text="Welcome to Leptos CSR" />

		// injects metadata in the <head> of the page
		<Meta charset="UTF-8" />
		<Meta name="viewport" content="width=device-width, initial-scale=1.0" />

		<Router>
			<Routes fallback=|| view! { <NotFound /> }>
				<Route path=path!("/") view=Home />
			</Routes>
		</Router>
	}
}
