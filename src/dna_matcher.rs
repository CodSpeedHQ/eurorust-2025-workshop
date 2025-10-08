/// Naive approach: Read the entire file as a string and filter lines
pub fn naive_dna_matcher<'a>(genome: &'a str, pattern: &str) -> Vec<&'a str> {
    let matcher = jetscii::Substring::new(pattern);
    let n_matcher = jetscii::Substring::new("\n");
    let mut position = 0;
    let mut indices = vec![];
    let indices = loop {
        let Some(idx) = matcher.find(&genome[position..]) else {
            // println!("done {indices:?}");
            break indices;
        };
        let mut newline = memchr::Memchr::new(b'\n', genome[position..position+idx].as_bytes());
        let start = newline.next_back().unwrap_or_default();
        let end = n_matcher.find(&genome[position+idx..]).unwrap_or_else(|| genome.len());
        // println!("Found match at {}-{}, idx {}, pos {}, start {}, end {}", position + start, position + end, idx, position, start, end);
        indices.push((position + start + 1, position + idx + end));
        position += idx + end;
    };
    indices.iter().map(|&(start, end)| &genome[start..end]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naive_matcher() {
        let test_genome = ">seq1\nACGTACGT\n>seq2\nAGTCCGTAAA\n>seq3\nGGGGGG";
        let pattern = "AGTCCGTA";
        let matches = naive_dna_matcher(test_genome, pattern);
        // println!("{:?}", matches);
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
        // println!("{:?}", matches);

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
