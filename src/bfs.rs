use bit_set::BitSet;
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
    let mut queue = Vec::new(); // Using Vec instead of VecDeque - intentionally inefficient!
    let mut result = Vec::new();

    queue.push(start);
    visited.insert(start);

    while !queue.is_empty() {
        // remove(0) is O(n) - this makes BFS slow!
        let node = queue.remove(0);
        result.push(node);

        if let Some(neighbors) = graph.adjacency.get(node) {
            for &neighbor in neighbors {
                if visited.insert(neighbor) {
                    queue.push(neighbor);
                }
            }
        }
    }

    result
}

pub fn bfs_optimized(graph: &Graph, start: usize) -> Vec<usize> {
    let mut visited = BitSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();

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
