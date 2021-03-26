use crate::collector;
use crate::nodedb::{self, Node};
use crate::NodeId;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use serde_json as json;
use std::collections::HashMap;
use crate::respondd::*;

pub fn generate_gatewaypath_graph(nodes: Vec<Node>) -> DiGraph<Node, u8> {
	let (mut graph, idt) = generate_base_graph(nodes);
	// add edges
	for idx in graph.node_indices() {
		let node = graph.node_weight(idx).unwrap();
		let nexthop: NodeId = node
			.last_response
			.as_object()
			.unwrap()
			.get("statistics")
			.unwrap()
			.as_object()
			.unwrap()
			.get("gateway_nexthop")
			.unwrap()
			.as_str()
			.unwrap()
			.to_string();

		if let Some(to_idx) = idt.get(&nexthop) {
			graph.add_edge(idx, to_idx.to_owned(), 1);
		}

	}

	graph
}

pub fn generate_batman_graph(nodes: Vec<Node>) -> DiGraph<Node, u8> {
	let (mut graph, idt) = generate_base_graph(nodes);
	// add edges
	for idx in graph.node_indices() {
		let node = graph.node_weight(idx).unwrap();
		let neighbors: Neighbours = json::from_value(
			node.last_response.as_object().unwrap().get("neighbours").unwrap().clone()
		).unwrap();

		for iface in neighbors.batadv.values() {
			for (neigh, vals) in iface.neighbours.iter() {
				let to_idx = idt.get(neigh).unwrap().clone();
				graph.add_edge(idx, to_idx, vals.tq);
			}
		}
	}

	graph
}

fn generate_base_graph(nodes: Vec<Node>) -> (DiGraph<Node, u8>, HashMap<NodeId, NodeIndex>) {
	let mut graph: DiGraph<Node, u8> = DiGraph::new();
	let mut idt: HashMap<NodeId, NodeIndex> = HashMap::new();

	// add nodes
	for node in &nodes {
		let idx = graph.add_node(node.clone());
		idt.insert(node.nodeid.clone(), idx);
	}

	(graph, idt)
}
