use std::collections::{HashSet, VecDeque};

/// A simple graph represented as an adjacency list
#[derive(Debug, Clone)]
pub struct Graph {
    /// adjacency[i] contains a list of nodes adjacent to node i
    pub adjacency: Vec<Vec<usize>>,
}

impl Graph {
    pub fn new(nodes: usize) -> Self {
        Graph {
            adjacency: vec![vec![]; nodes],
        }
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.adjacency[from].push(to);
    }

    pub fn num_nodes(&self) -> usize {
        self.adjacency.len()
    }
}

/// Naive BFS implementation using Vec as a queue (intentionally slow)
/// Returns the order in which nodes were visited
pub fn bfs_naive(graph: &Graph, start: usize) -> Vec<usize> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new(); // Using Vec instead of VecDeque - intentionally inefficient!
    let mut result = Vec::new();

    queue.reserve(graph.num_nodes());
    result.reserve(graph.num_nodes());
    visited.reserve(graph.num_nodes());

    queue.push_back(start);
    visited.insert(start);

    while let Some(node) = queue.pop_front() {
        result.push(node);

        if let Some(neighbors) = graph.adjacency.get(node) {
            for &neighbor in neighbors {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
    }

    result
}

/// Helper function to generate a random graph for benchmarking
pub fn generate_graph(nodes: usize) -> Graph {
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut graph = Graph::new(nodes);

    for i in 0..nodes {
        for _ in 0..10 {
            let target = rng.gen_range(0..nodes);
            if target != i {
                graph.add_edge(i, target);
            }
        }
    }

    graph
}
