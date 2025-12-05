//! Graph data structures for input to the force graph component.

/// A node in the graph.
#[derive(Clone, Debug)]
pub struct GraphNode {
	/// Unique identifier for this node. Used to reference nodes in links.
	pub id: String,
	/// Optional display label. Labeled nodes are rendered larger.
	pub label: Option<String>,
	/// Optional CSS color override (e.g., "#ff0000" or "rgb(255, 0, 0)").
	/// If not set, color is derived from the theme palette based on `group`.
	pub color: Option<String>,
	/// Optional group index for palette-based coloring.
	pub group: Option<u32>,
}

/// A directed edge between two nodes.
#[derive(Clone, Debug)]
pub struct GraphLink {
	/// Source node ID.
	pub source: String,
	/// Target node ID.
	pub target: String,
}

/// Complete graph data: nodes and links.
#[derive(Clone, Debug, Default)]
pub struct GraphData {
	pub nodes: Vec<GraphNode>,
	pub links: Vec<GraphLink>,
}
