use itertools::*;

pub fn dna_matcher_api(genome: &str, pattern: &str) -> Vec<String> {
    optimized_dna_matcher_impl(genome, pattern)
}

/// Naive approach: Read the entire file as a string and filter lines
#[allow(dead_code)]
fn naive_dna_matcher_impl(genome: &str, pattern: &str) -> Vec<String> {
    genome
        .lines()
        .filter(|line| !line.starts_with('>')) // Skip headers
        .filter(|line| line.contains(pattern))
        .map(|s| s.to_string())
        .collect()
}

fn optimized_dna_matcher_impl(genome: &str, pattern: &str) -> Vec<String> {
    std::iter::once(usize::MAX)
        .chain(genome.as_bytes().iter().positions(|&c| c == b'\n'))
        .chain(std::iter::once(genome.len()))
        .tuple_windows()
        .filter_map(|(start, end)| {
            let line = if start == usize::MAX {
                &genome[..end]
            } else {
                &genome[start + 1..end]
            };
            if line.len() == 0 || line.as_bytes()[0] == b'>' {
                None
            } else {
                Some(line)
            }
        })
        .filter(|line| line.contains(pattern))
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matcher() {
        let test_genome = ">seq1\nACGTACGT\n>seq2\nAGTCCGTAAA\n>seq3\nGGGGGG";
        let pattern = "AGTCCGTA";
        let matches = dna_matcher_api(test_genome, pattern);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], "AGTCCGTAAA");
    }

    #[test]
    fn test_matcher_on_genome_file() {
        // Read the actual genome.fasta file
        let genome = std::fs::read_to_string("genome.fasta")
            .expect("Failed to read genome.fasta\n\n Make sure to run 'cargo run --release --bin generate_fasta'");
        let pattern = "AGTCCGTA";

        let matches = dna_matcher_api(&genome, pattern);

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
