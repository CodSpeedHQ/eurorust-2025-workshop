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

pub fn memchr_search_bytes_parallel(genome: &[u8], pattern: &[u8]) -> Vec<Vec<u8>> {
    use rayon::prelude::*;
    use std::collections::HashSet;
    use std::sync::Mutex;

    let chunk_size = genome.len() / rayon::current_num_threads().max(1);
    let chunk_size = chunk_size.max(1024 * 1024); // At least 1MB per chunk

    let seen = Mutex::new(HashSet::new());

    // Split genome into chunks at newline boundaries
    let mut chunk_starts = vec![0];
    let mut pos = chunk_size;
    while pos < genome.len() {
        if let Some(newline_pos) = memchr::memchr(b'\n', &genome[pos..]) {
            chunk_starts.push(pos + newline_pos + 1);
            pos += newline_pos + 1 + chunk_size;
        } else {
            break;
        }
    }
    chunk_starts.push(genome.len());

    chunk_starts
        .par_windows(2)
        .flat_map(|window| {
            let start = window[0];
            let end = window[1];
            let chunk = &genome[start..end];

            let mut local_results = Vec::new();

            memchr::memmem::find_iter(chunk, pattern).for_each(|match_pos| {
                // Walk back to find the start of the line
                let line_start = memchr::memrchr(b'\n', &chunk[..match_pos])
                    .map(|pos| pos + 1)
                    .unwrap_or(0);

                // Check if it's a header
                if chunk.get(line_start) == Some(&b'>') {
                    return;
                }

                // Extract the full sequence line containing the match
                let line_end = memchr::memchr(b'\n', &chunk[line_start..])
                    .map(|pos| line_start + pos)
                    .unwrap_or(chunk.len());

                let line = &chunk[line_start..line_end];

                // Check if we've seen this line before
                let mut seen_guard = seen.lock().unwrap();
                if seen_guard.insert(line) {
                    local_results.push(line.to_vec());
                }
            });

            local_results
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

    #[test]
    fn test_parallel_matcher_on_genome_file() {
        // Read the actual genome.fasta file
        let genome = std::fs::read("genome.fasta")
            .expect("Failed to read genome.fasta\n\n Make sure to run 'cargo run --release --bin generate_fasta'");
        let pattern = b"AGTCCGTA";

        let matches = memchr_search_bytes_parallel(&genome, pattern);

        // With fixed seed (42), we should always get exactly 4927 matches
        assert_eq!(
            matches.len(),
            4927,
            "Expected 4927 matches with seed 42, found {}",
            matches.len()
        );

        println!(
            "✓ Found {} sequences containing pattern '{}' (parallel)",
            matches.len(),
            String::from_utf8_lossy(pattern)
        );
    }
}
