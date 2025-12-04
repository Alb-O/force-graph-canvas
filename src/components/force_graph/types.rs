#[derive(Clone, Debug)]
pub struct GraphNode {
	pub id: String,
	pub label: Option<String>,
	pub color: Option<String>,
	pub group: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct GraphLink {
	pub source: String,
	pub target: String,
}

#[derive(Clone, Debug, Default)]
pub struct GraphData {
	pub nodes: Vec<GraphNode>,
	pub links: Vec<GraphLink>,
}
