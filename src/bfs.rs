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
    let mut visited = Vec::from_iter((0..graph.num_nodes()).map(|_| false));
    let mut queue = VecDeque::with_capacity(1024);
    let mut result = Vec::with_capacity(graph.num_nodes());

    queue.push_back(start);
    visited[start] = true;

    while !queue.is_empty() {
        let node = queue.pop_front().unwrap();
        result.push(node);

        if let Some(neighbors) = graph.adjacency.get(node) {
            for &neighbor in neighbors {
                if !visited[neighbor] {
                    visited[neighbor] = true;
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
