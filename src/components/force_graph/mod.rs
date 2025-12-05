//! Force-directed graph visualization component.
//!
//! Renders an interactive force-directed graph on an HTML canvas with:
//! - Physics-based node positioning via force simulation
//! - Pan, zoom, and node dragging interactions
//! - Smooth highlight transitions on hover
//! - Configurable theming and visual scaling
//!
//! # Example
//!
//! ```ignore
//! use force_graph_canvas::{ForceGraphCanvas, GraphData, GraphNode, GraphLink};
//!
//! let data = GraphData {
//!     nodes: vec![
//!         GraphNode { id: "a".into(), label: Some("Node A".into()), .. },
//!         GraphNode { id: "b".into(), label: Some("Node B".into()), .. },
//!     ],
//!     links: vec![
//!         GraphLink { source: "a".into(), target: "b".into() },
//!     ],
//! };
//!
//! view! { <ForceGraphCanvas data=data.into() fullscreen=true /> }
//! ```

mod component;
mod particles;
mod render;
pub mod scale;
mod state;
pub mod theme;
mod types;

pub use component::ForceGraphCanvas;
pub use theme::Theme;
pub use types::{GraphData, GraphLink, GraphNode};
