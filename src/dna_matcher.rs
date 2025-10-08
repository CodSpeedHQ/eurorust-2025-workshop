use rayon::prelude::*;
use memchr::memmem;

pub fn exported_dna_matcher(genome: &str, pattern: &str) -> Vec<String> {
    chunked_dna_matcher(genome, pattern)
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

/// Chunked approach: Process genome in parallel byte chunks
fn chunked_dna_matcher(genome: &str, pattern: &str) -> Vec<String> {
    let pattern_bytes = pattern.as_bytes();
    let genome_bytes = genome.as_bytes();
    let finder = memmem::Finder::new(pattern_bytes);

    // Chunk size: balance between parallelism and overhead
    // Aim for ~1000 lines per chunk, with typical DNA line length of 60-80 chars
    let chunk_size = 64 * 1024; // 64KB per chunk
    let total_len = genome_bytes.len();

    // Find chunk boundaries that align with line boundaries
    let mut chunk_starts = vec![0];
    let mut pos = chunk_size;

    while pos < total_len {
        // Find the next newline after pos
        let search_start = pos;
        let search_end = std::cmp::min(pos + 1024, total_len); // Look ahead up to 1KB for newline

        if let Some(newline_offset) = memchr::memchr(b'\n', &genome_bytes[search_start..search_end]) {
            chunk_starts.push(search_start + newline_offset + 1);
            pos = search_start + newline_offset + 1 + chunk_size;
        } else {
            // No newline found, just use the current position
            chunk_starts.push(pos);
            pos += chunk_size;
        }
    }
    chunk_starts.push(total_len);

    // Process chunks in parallel
    let matches: Vec<String> = (0..chunk_starts.len() - 1)
        .into_par_iter()
        .flat_map(|i| {
            let chunk_start = chunk_starts[i];
            let chunk_end = chunk_starts[i + 1];
            let chunk = &genome_bytes[chunk_start..chunk_end];

            let mut local_matches = Vec::new();
            let mut line_start = 0;

            // Use memchr_iter for faster newline finding
            for newline_pos in memchr::memchr_iter(b'\n', chunk) {
                let line = &chunk[line_start..newline_pos];
                line_start = newline_pos + 1;

                // Skip headers and empty lines
                if !line.is_empty() && line[0] != b'>' {
                    // Use memmem for fast substring search
                    if finder.find(line).is_some() {
                        // SAFETY: DNA sequences are ASCII-only, so we can skip UTF-8 validation
                        let line_str = unsafe { std::str::from_utf8_unchecked(line) };
                        local_matches.push(line_str.to_string());
                    }
                }
            }

            // Handle last line if chunk doesn't end with newline
            if line_start < chunk.len() {
                let line = &chunk[line_start..];
                if !line.is_empty() && line[0] != b'>' {
                    if finder.find(line).is_some() {
                        // SAFETY: DNA sequences are ASCII-only, so we can skip UTF-8 validation
                        let line_str = unsafe { std::str::from_utf8_unchecked(line) };
                        local_matches.push(line_str.to_string());
                    }
                }
            }

            local_matches
        })
        .collect();

    matches
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
