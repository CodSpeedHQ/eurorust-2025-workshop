/// Naive approach: Read the entire file as a string and filter lines
pub fn naive_dna_matcher(genome: &str, pattern: &str) -> Vec<String> {
    return memchr_search(genome, pattern);
}

pub fn memchr_search(genome: &str, pattern: &str) -> Vec<String> {
    memchr_search_bytes(genome.as_bytes(), pattern.as_bytes())
        .into_iter()
        .map(|bytes| String::from_utf8(bytes).expect("Invalid UTF-8"))
        .collect()
}

pub fn memchr_search_bytes(genome: &[u8], pattern: &[u8]) -> Vec<Vec<u8>> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    memchr::memmem::find_iter(genome, pattern)
        .filter_map(|match_pos| {
            // Walk back to find the start of the line
            let start = memchr::memrchr(b'\n', &genome[..match_pos])
                .map(|pos| pos + 1)
                .unwrap_or(0);

            // Check if it's a header
            if genome.get(start) == Some(&b'>') {
                return None;
            }

            // Extract the full sequence line containing the match
            let end = memchr::memchr(b'\n', &genome[start..])
                .map(|pos| start + pos)
                .unwrap_or(genome.len());

            let line = &genome[start..end];

            // Only include unique lines
            if seen.insert(line) {
                Some(line.to_vec())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naive_matcher() {
        let test_genome = ">seq1\nACGTACGT\n>seq2\nAGTCCGTAAA\n>seq3\nGGGGGG";
        let pattern = "AGTCCGTA";
        let matches = naive_dna_matcher(test_genome, pattern);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], "AGTCCGTAAA");
    }

    #[test]
    fn test_naive_matcher_on_genome_file() {
        // Read the actual genome.fasta file
        let genome = std::fs::read_to_string("genome.fasta")
            .expect("Failed to read genome.fasta\n\n Make sure to run 'cargo run --release --bin generate_fasta'");
        let pattern = "AGTCCGTA";

        let matches = naive_dna_matcher(&genome, pattern);

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
