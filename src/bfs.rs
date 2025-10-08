use std::collections::HashSet;
use arraydeque::ArrayDeque;

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

/// Naive BFS implementation using ArrayDeque as a queue
/// Returns the order in which nodes were visited
pub fn bfs_naive(graph: &Graph, start: usize) -> Vec<usize> {
    let mut visited = HashSet::new();
    let mut queue: ArrayDeque<usize, 16384> = ArrayDeque::new(); // Using ArrayDeque with capacity of 16384
    let mut result = Vec::new();

    queue.push_back(start).ok();
    visited.insert(start);

    while !queue.is_empty() {
        let node = queue.pop_front().unwrap();
        result.push(node);

        if let Some(neighbors) = graph.adjacency.get(node) {
            for &neighbor in neighbors {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor).ok();
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
