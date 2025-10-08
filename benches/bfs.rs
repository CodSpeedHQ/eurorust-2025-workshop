use divan::Bencher;
use eurorust_2025_workshop::bfs::{bfs_optimized, generate_graph};

fn main() {
    divan::main();
}

#[divan::bench]
fn bfs_small_graph(bencher: Bencher) {
    let graph = generate_graph(100);

    bencher.bench_local(|| {
        let result = divan::black_box(bfs_optimized(divan::black_box(&graph), divan::black_box(0)));

        assert!(!result.is_empty(), "BFS result should not be empty");
        assert!(
            result.len() <= 100,
            "BFS result should not exceed graph size"
        );
        assert_eq!(result[0], 0, "First node should be the start node");
        assert_eq!(result[10], 25, "Node at position 10 should be 25");
        assert_eq!(result[50], 53, "Node at position 50 should be 53");
    });
}

#[divan::bench]
fn bfs_medium_graph(bencher: Bencher) {
    let graph = generate_graph(1000);

    bencher.bench_local(|| {
        let result = divan::black_box(bfs_optimized(divan::black_box(&graph), divan::black_box(0)));

        assert!(!result.is_empty(), "BFS result should not be empty");
        assert!(
            result.len() <= 1000,
            "BFS result should not exceed graph size"
        );
        assert_eq!(result[0], 0, "First node should be the start node");
        assert_eq!(result[100], 428, "Node at position 100 should be 428");
        assert_eq!(result[500], 397, "Node at position 500 should be 397");
    });
}

#[divan::bench]
fn bfs_large_graph(bencher: Bencher) {
    let graph = generate_graph(10000);

    bencher.bench_local(|| {
        let result = divan::black_box(bfs_optimized(divan::black_box(&graph), divan::black_box(0)));

        assert!(!result.is_empty(), "BFS result should not be empty");
        assert!(
            result.len() <= 10000,
            "BFS result should not exceed graph size"
        );
        assert_eq!(result[0], 0, "First node should be the start node");
        assert_eq!(result[1000], 7575, "Node at position 1000 should be 7575");
        assert_eq!(result[2500], 5949, "Node at position 2500 should be 5949");
    });
}
