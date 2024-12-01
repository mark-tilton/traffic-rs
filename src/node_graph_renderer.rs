use bevy::prelude::*;

use crate::node_graph::NodeGraph;

#[derive(Resource, Default)]
pub struct NodeGraphRenderer {
    // The id of the vehicle which is highlighted on the screen
    pub highlighted_vehicle_id: Option<usize>,
    // The index of the path in shortest_path_map which is highlighted on the screen
    pub highlighted_path_index: Option<(usize, usize)>,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct HighlightedEdgeGizmos {}

pub fn configure_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (highlighted_edge_gizmo_config, _) = config_store.config_mut::<HighlightedEdgeGizmos>();
    highlighted_edge_gizmo_config.line_width = 5.0;
}

pub fn show_node_graph(
    node_graph: Res<NodeGraph>,
    node_graph_renderer: Res<NodeGraphRenderer>,
    mut gizmos: Gizmos,
    mut highlighted_edge_gizmos: Gizmos<HighlightedEdgeGizmos>,
) {
    let node_radius = 0.5;
    // Draw nodes different colors based on their types
    for (i, node) in node_graph.nodes.iter().enumerate() {
        let color = if node_graph.source_nodes.contains(&i) {
            Color::srgb(0.1, 0.9, 0.1)
        } else if node_graph.dest_nodes.contains(&i) {
            Color::srgb(0.9, 0.1, 0.1)
        } else {
            Color::srgb(0.1, 0.1, 0.9)
        };
        gizmos.sphere(node.position, Quat::IDENTITY, node_radius, color);
    }

    let highlighted_path: Option<&Vec<usize>> = match node_graph_renderer.highlighted_path_index {
        Some(highlighted_path_index) => node_graph.shortest_path_map.get(&highlighted_path_index),
        None => None,
    };

    // Draw edges as arrows while leaving space for the node.
    for ((source, dest), _) in node_graph.edges.iter() {
        let source_pos = node_graph.nodes[*source].position;
        let dest_pos = node_graph.nodes[*dest].position;
        let dest_to_src = dest_pos - source_pos;
        let arrow_start = source_pos + dest_to_src.normalize() * node_radius;
        let arrow_end = source_pos + dest_to_src.normalize() * (dest_to_src.length() - node_radius);

        if let Some(highlighted_path) = highlighted_path {
            if NodeGraph::is_edge_in_path(*source, *dest, highlighted_path) {
                highlighted_edge_gizmos.arrow(arrow_start, arrow_end, Color::srgb(1., 0., 1.));
                continue;
            }
        }
        gizmos.arrow(arrow_start, arrow_end, Color::srgb(1., 1., 1.));
    }
}
