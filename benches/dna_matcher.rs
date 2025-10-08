use eurorust_2025_workshop::dna_matcher::*;

fn main() {
    divan::main();
}

#[divan::bench(sample_count = 2, sample_size = 3)]
fn dna_matcher() {
    use bytes::Bytes;
    use memmap2::Mmap;
    use std::fs::File;
    use std::ops::Deref;

    let file = File::open("genome.fasta").expect(
        "Failed to read genome.fasta\n\n Make sure to run 'cargo run --release --bin generate_fasta'",
    );
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    let genome = Bytes::from_owner(mmap);
    let pattern = "AGTCCGTA";

    let matches = divan::black_box(dna_matcher_api(
        divan::black_box(genome.deref()),
        divan::black_box(pattern),
    ));

    assert!(
        matches.len() == 4927,
        "Expected 4927 matches, found {}",
        matches.len()
    );
}
