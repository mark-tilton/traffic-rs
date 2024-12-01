use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;

#[derive(Clone)]
pub struct Node {
    pub position: Vec3,
}

#[derive(Default)]
pub struct Edge {
    pub priority: bool,
}

#[derive(Resource)]
pub struct NodeGraph {
    pub nodes: Vec<Node>,
    pub edges: HashMap<(usize, usize), Edge>,
    // Source nodes are nodes that have no other nodes pointing to them
    pub source_nodes: HashSet<usize>,
    // Destination nodes are nodes that don't have any nodes leading from them
    pub dest_nodes: HashSet<usize>,
    // A convenient data structure for navigating forward through the graph
    pub node_map: HashMap<usize, HashSet<usize>>,
    // A convenient data structure for navigating backward through the graph
    pub reverse_node_map: HashMap<usize, HashSet<usize>>,
    // Stores the shortest path for a given source/destination node pair
    pub shortest_path_map: HashMap<(usize, usize), Vec<usize>>,
    // Stores which vehicle has a given node reserved
    pub node_reservation_map: HashMap<usize, usize>,
}

impl NodeGraph {
    // Creates a four way intersection with the following structure
    //          2     3
    //          |     ^
    //          V     |
    //    4<---10<----11<----6
    //          | \ / ^
    //          |  X  |
    //          V / \ |
    //    5---->8---->9----->7
    //          |     ^
    //          V     |
    //          0     1
    pub fn create_nightmare() -> Self {
        // Bevy uses a right handed y-up coordinate system
        // This means that the forward vector is -z
        let node_positions = [
            // Bottom
            Vec3::new(-1., 0., 10.),
            Vec3::new(1., 0., 10.),
            // Top
            Vec3::new(-1., 0., -10.),
            Vec3::new(1., 0., -10.),
            // Left
            Vec3::new(-10., 0., -1.),
            Vec3::new(-10., 0., 1.),
            // Right
            Vec3::new(10., 0., -1.),
            Vec3::new(10., 0., 1.),
            // Intersection
            Vec3::new(-1., 0., 1.),
            Vec3::new(1., 0., 1.),
            Vec3::new(-1., 0., -1.),
            Vec3::new(1., 0., -1.),
        ];
        let nodes = node_positions.map(|position| Node { position }).to_vec();
        let edges = HashMap::from([
            // Sources to the intersection
            ((1, 9), Edge { priority: false }),
            ((2, 10), Edge { priority: false }),
            ((6, 11), Edge { priority: false }),
            ((5, 8), Edge { priority: false }),
            // Intersection out to destinations
            ((9, 7), Edge { priority: false }),
            ((11, 3), Edge { priority: false }),
            ((10, 4), Edge { priority: false }),
            ((8, 0), Edge { priority: false }),
            // Intersection to intersection
            ((9, 11), Edge { priority: false }),
            ((9, 10), Edge { priority: false }),
            ((11, 10), Edge { priority: false }),
            ((11, 8), Edge { priority: false }),
            ((10, 8), Edge { priority: false }),
            ((10, 9), Edge { priority: false }),
            ((8, 9), Edge { priority: false }),
            ((8, 11), Edge { priority: false }),
        ]);
        Self::new(nodes, edges)
    }

    // Creates a four way intersection with the following structure
    //          2     3
    //          ^     |
    //          |     V
    //    4--->10<----11---->6
    //          |     ^
    //          |     |
    //          V     |
    //    5<----8---->9<-----7
    //          ^     |
    //          |     V
    //          0     1
    pub fn create_roundabout() -> Self {
        // Bevy uses a right handed y-up coordinate system
        // This means that the forward vector is -z
        let node_positions = [
            // Bottom
            Vec3::new(-5., 0., 10.),
            Vec3::new(5., 0., 10.),
            // Top
            Vec3::new(-5., 0., -10.),
            Vec3::new(5., 0., -10.),
            // Left
            Vec3::new(-10., 0., -5.),
            Vec3::new(-10., 0., 5.),
            // Right
            Vec3::new(10., 0., -5.),
            Vec3::new(10., 0., 5.),
            // Intersection In
            Vec3::new(-2., 0., 3.),
            Vec3::new(3., 0., 2.),
            Vec3::new(-3., 0., -2.),
            Vec3::new(2., 0., -3.),
            // Intersection Out
            Vec3::new(-3., 0., 2.),
            Vec3::new(2., 0., 3.),
            Vec3::new(-2., 0., -3.),
            Vec3::new(3., 0., -2.),
        ];
        let nodes = node_positions.map(|position| Node { position }).to_vec();
        let edges = HashMap::from([
            // Sources to the intersection
            ((7, 9), Edge { priority: false }),
            ((3, 11), Edge { priority: false }),
            ((4, 10), Edge { priority: false }),
            ((0, 8), Edge { priority: false }),
            // Intersection out to destinations
            ((13, 1), Edge { priority: true }),
            ((14, 2), Edge { priority: true }),
            ((15, 6), Edge { priority: true }),
            ((12, 5), Edge { priority: true }),
            // Rotary Connections
            ((12, 8), Edge { priority: true }),
            ((13, 9), Edge { priority: true }),
            ((14, 10), Edge { priority: true }),
            ((15, 11), Edge { priority: true }),
            ((9, 15), Edge { priority: true }),
            ((11, 14), Edge { priority: true }),
            ((10, 12), Edge { priority: true }),
            ((8, 13), Edge { priority: true }),
        ]);
        Self::new(nodes, edges)
    }

    pub fn new(nodes: Vec<Node>, edges: HashMap<(usize, usize), Edge>) -> Self {
        // Automatically classify nodes as source, or destination nodes based
        // on edge directions.
        let mut source_nodes = HashSet::from_iter(0..nodes.len());
        let mut dest_nodes = HashSet::from_iter(0..nodes.len());
        let mut node_map: HashMap<usize, HashSet<usize>> = HashMap::new();
        for ((source, dest), _) in edges.iter() {
            dest_nodes.remove(source);
            source_nodes.remove(dest);
            node_map.entry(*source).or_default().insert(*dest);
        }
        let reverse_node_map = calculate_reverse_node_map(&node_map);
        let shortest_path_map =
            calculate_shortest_path_map(&source_nodes, &dest_nodes, &node_map, &reverse_node_map);
        NodeGraph {
            nodes,
            edges,
            source_nodes,
            dest_nodes,
            node_map,
            reverse_node_map,
            shortest_path_map,
            node_reservation_map: HashMap::new(),
        }
    }

    pub fn is_edge_in_path(source_node: usize, dest_node: usize, path: &Vec<usize>) -> bool {
        let Some(source_index) = path.iter().position(|x| x == &source_node) else {
            return false;
        };

        let Some(dest_index) = path.iter().position(|x| x == &dest_node) else {
            return false;
        };

        return dest_index == source_index + 1;
    }
}

fn calculate_shortest_path_map(
    source_nodes: &HashSet<usize>,
    dest_nodes: &HashSet<usize>,
    node_map: &HashMap<usize, HashSet<usize>>,
    reverse_node_map: &HashMap<usize, HashSet<usize>>,
) -> HashMap<(usize, usize), Vec<usize>> {
    let mut shortest_path_map = HashMap::new();

    for source_node in source_nodes {
        for dest_node in dest_nodes {
            if let Some(shortest_path) =
                calculate_shortest_path(*source_node, *dest_node, node_map, reverse_node_map)
            {
                shortest_path_map.insert((*source_node, *dest_node), shortest_path);
            }
        }
    }

    return shortest_path_map;
}

fn calculate_reverse_node_map(
    node_map: &HashMap<usize, HashSet<usize>>,
) -> HashMap<usize, HashSet<usize>> {
    let mut reverse_node_map: HashMap<usize, HashSet<usize>> = HashMap::new();

    for (node, connections) in node_map {
        for connection in connections {
            reverse_node_map
                .entry(*connection)
                .or_default()
                .insert(*node);
        }
    }

    return reverse_node_map;
}

fn calculate_shortest_path(
    source_node: usize,
    dest_node: usize,
    node_map: &HashMap<usize, HashSet<usize>>,
    reverse_node_map: &HashMap<usize, HashSet<usize>>,
) -> Option<Vec<usize>> {
    let distance_map = calculate_distance_map(source_node, node_map);

    // if the destination doesn't have a distance then it must be unreachable
    if !distance_map.contains_key(&dest_node) {
        return None;
    }

    // find the shortest path by traversing backwards from destination back to the source
    let mut shortest_path = Vec::new();
    let mut node = dest_node;
    shortest_path.push(node);
    loop {
        let connections = reverse_node_map
            .get(&node)
            .expect("Node not contained in reverse node map");

        // Find the next node by sorting the available connections by their value in the distance map
        node = *connections
            .iter()
            .filter(|x| distance_map.contains_key(x))
            .min_by_key(|x| distance_map.get(x))
            .expect("Error calculating next node");

        shortest_path.push(node);

        if node == source_node {
            break;
        }
    }

    // Nodes were added in reverse order, need to reverse collection
    shortest_path.reverse();

    return Some(shortest_path);
}

fn calculate_distance_map(
    source_node: usize,
    node_map: &HashMap<usize, HashSet<usize>>,
) -> HashMap<usize, usize> {
    let mut distance_map: HashMap<usize, usize> = HashMap::new();
    let mut queue: VecDeque<usize> = VecDeque::new();

    distance_map.insert(source_node, 0);
    queue.push_back(source_node);

    // Do a breadth first search of the tree
    loop {
        let Some(node) = queue.pop_front() else {
            break;
        };

        let distance = *distance_map
            .get(&node)
            .expect("Queued node should have a distance");
        let Some(connections) = node_map.get(&node) else {
            continue;
        };

        for connection in connections {
            if !distance_map.contains_key(connection) {
                distance_map.insert(*connection, distance + 1);
                queue.push_back(*connection);
            }
        }
    }
    return distance_map;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_distance_map_produces_expected_values() {
        // commented out test cases are for uturn scenarios which don't seem valid
        let expected_values = vec![
            (1, 7, vec![1, 9, 7]),
            (1, 3, vec![1, 9, 11, 3]),
            (1, 4, vec![1, 9, 10, 4]),
            // (1, 0, vec![1, 9, 10, 8, 0]),
            (6, 3, vec![6, 11, 3]),
            (6, 4, vec![6, 11, 10, 4]),
            (6, 0, vec![6, 11, 8, 0]),
            // (6, 7, vec![6, 11, 8, 9, 7]),
            (2, 4, vec![2, 10, 4]),
            (2, 0, vec![2, 10, 8, 0]),
            (2, 7, vec![2, 10, 9, 7]),
            // (2, 3, vec![2, 10, 9, 11, 3]),
            (5, 0, vec![5, 8, 0]),
            (5, 7, vec![5, 8, 9, 7]),
            (5, 3, vec![5, 8, 11, 3]),
            // (5, 4, vec![5, 8, 11, 10, 4]),
        ];
        let graph = NodeGraph::create_nightmare();

        for (source_node, dest_node, expected_path) in expected_values {
            let shortest_path = graph
                .shortest_path_map
                .get(&(source_node, dest_node))
                .unwrap();
            assert_eq!(
                expected_path, *shortest_path,
                "Input of ({}, {}) produced an unexpected shortest path",
                source_node, dest_node
            );
        }
    }
}
