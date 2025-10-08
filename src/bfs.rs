use core::hash::{BuildHasherDefault, Hasher};
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

pub(crate) type BuildNoHashHasher = BuildHasherDefault<NoHashHasher>;

#[derive(Default)]
pub(crate) struct NoHashHasher(u64);

impl Hasher for NoHashHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        unreachable!("Should not be used")
    }
    fn write_u8(&mut self, _: u8) {
        unreachable!("Should not be used")
    }
    fn write_u16(&mut self, _: u16) {
        unreachable!("Should not be used")
    }
    fn write_u32(&mut self, _: u32) {
        unreachable!("Should not be used")
    }
    fn write_u64(&mut self, _: u64) {
        unreachable!("Should not be used")
    }
    fn write_usize(&mut self, n: usize) {
        self.0 = n as u64;
    }
    fn write_i8(&mut self, _: i8) {
        unreachable!("Should not be used")
    }
    fn write_i16(&mut self, _: i16) {
        unreachable!("Should not be used")
    }
    fn write_i32(&mut self, _: i32) {
        unreachable!("Should not be used")
    }
    fn write_i64(&mut self, _: i64) {
        unreachable!("Should not be used")
    }
    fn write_isize(&mut self, _: isize) {
        unreachable!("Should not be used")
    }
}

/// Naive BFS implementation using Vec as a queue (intentionally slow)
/// Returns the order in which nodes were visited
pub fn bfs_naive(graph: &Graph, start: usize) -> Vec<usize> {
    let mut visited = HashSet::with_capacity_and_hasher(1024, BuildNoHashHasher::new());
    let mut queue = VecDeque::new(); // Using Vec instead of VecDeque - intentionally inefficient!
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
