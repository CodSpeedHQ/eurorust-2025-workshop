use rayon::prelude::*;


pub fn exported_dna_matcher(genome: &str, pattern: &str) -> Vec<String> {
    naive_dna_matcher(genome, pattern)
}

/// Naive approach: Read the entire file as a string and filter lines
 fn naive_dna_matcher(genome: &str, pattern: &str) -> Vec<String> {
    genome
        .par_lines()
        .filter(|line| !line.starts_with('>')) // Skip headers
        .filter(|line| line.contains(pattern))
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naive_matcher() {
        let test_genome = ">seq1\nACGTACGT\n>seq2\nAGTCCGTAAA\n>seq3\nGGGGGG";
        let pattern = "AGTCCGTA";
        let matches = exported_dna_matcher(test_genome, pattern);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], "AGTCCGTAAA");
    }

    #[test]
    fn test_naive_matcher_on_genome_file() {
        // Read the actual genome.fasta file
        let genome = std::fs::read_to_string("genome.fasta")
            .expect("Failed to read genome.fasta\n\n Make sure to run 'cargo run --release --bin generate_fasta'");
        let pattern = "AGTCCGTA";

        let matches = exported_dna_matcher(&genome, pattern);

        // With fixed seed (42), we should always get exactly 4927 matches
        assert_eq!(
            matches.len(),
            4927,
            "Expected 4927 matches with seed 42, found {}",
            matches.len()
        );

        println!(
            "✓ Found {} sequences containing pattern '{}'",
            matches.len(),
            pattern
        );
    }
}
