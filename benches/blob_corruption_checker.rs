use divan::Bencher;
use eurorust_2025_workshop::blob_corruption_checker::find_corruptions_sequential;

fn main() {
    divan::main();
}

#[divan::bench(sample_count = 3, sample_size = 5)]
fn corruption_check(bencher: Bencher) {
    bencher.bench_local(|| {
        let corruptions = divan::black_box(find_corruptions_sequential(
            "reference.bin",
            "corrupted.bin",
            1024, // 1KB chunks
        ));

        assert_eq!(corruptions.len(), 50, "Should find 50 corruptions");

        // All corruptions should be 1KB aligned
        for corruption in &corruptions {
            assert_eq!(corruption.offset % 1024, 0, "Corruption offset should be 1KB aligned");
            assert_eq!(corruption.length % 1024, 0, "Corruption length should be multiple of 1KB");
        }

        // Check specific corruptions
        assert_eq!(corruptions[0].offset, 14801920, "First corruption offset");
        assert_eq!(corruptions[0].length, 2048, "First corruption length");
        assert_eq!(corruptions[25].offset, 243891200, "Middle corruption offset");
        assert_eq!(corruptions[25].length, 4096, "Middle corruption length");
        assert_eq!(corruptions[49].offset, 507871232, "Last corruption offset");
        assert_eq!(corruptions[49].length, 5120, "Last corruption length");
    });
}
