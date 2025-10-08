use eurorust_2025_workshop::dna_matcher::*;

fn main() {
    divan::main();
}

#[divan::bench(sample_count = 2, sample_size = 3)]
fn dna_matcher() {
    let genome = std::fs::read_to_string("genome.fasta").expect(
        "Failed to read genome.fasta\n\n Make sure to run 'cargo run --release --bin generate_fasta'",
    );
    let pattern = "AGTCCGTA";

    let matches = divan::black_box(naive_dna_matcher(
        divan::black_box(&genome),
        divan::black_box(pattern),
    ));

    assert!(
        matches.len() == 4927,
        "Expected 4927 matches, found {}",
        matches.len()
    );
}

#[divan::bench(sample_count = 2, sample_size = 3)]
fn dna_matcher_memchr() {
    let genome = std::fs::read_to_string("genome.fasta").expect(
        "Failed to read genome.fasta\n\n Make sure to run 'cargo run --release --bin generate_fasta'",
    );
    let pattern = "AGTCCGTA";

    let matches = divan::black_box(memchr_search(
        divan::black_box(&genome),
        divan::black_box(pattern),
    ));

    assert!(
        matches.len() == 4927,
        "Expected 4927 matches, found {}",
        matches.len()
    );
}

#[divan::bench(sample_count = 2, sample_size = 3)]
fn dna_matcher_mmap() {
    let file = std::fs::File::open("genome.fasta").expect(
        "Failed to read genome.fasta\n\n Make sure to run 'cargo run --release --bin generate_fasta'",
    );
    let mmap = unsafe { memmap2::Mmap::map(&file).expect("Failed to mmap genome.fasta") };
    let pattern = b"AGTCCGTA";

    let matches = divan::black_box(memchr_search_bytes(
        divan::black_box(&mmap),
        divan::black_box(pattern),
    ));

    assert!(
        matches.len() == 4927,
        "Expected 4927 matches, found {}",
        matches.len()
    );
}
