use itertools::Itertools;
use jetscii::{ByteSubstring, bytes};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// Naive approach: Read the entire file as a string and filter lines
pub fn naive_dna_matcher<'a>(genome: &'a str, pattern: &'a str) -> Vec<&'a str> {
    let genome = genome.as_bytes();
    let pattern = pattern.as_bytes();
    let searcher = ByteSubstring::new(pattern);
    split_lines(genome)
        .into_iter()
        .collect_vec()
        .par_iter()
        .filter(|line| line.first() != Some(&b'>'))
        .filter(|line| searcher.find(line).is_some())
        .map(|s| unsafe { str::from_utf8_unchecked(s) })
        .collect()
}

fn split_lines(text: &[u8]) -> Vec<&[u8]> {
    let newlines = bytes!('\n');
    let mut offset = 0;
    let mut result = Vec::with_capacity(128);
    while offset < text.len() {
        if let Some(next_offset_delta) = newlines.find(&text[offset..]) {
            let next_offset = offset + next_offset_delta;
            // println!("offset={offset} next_offset={next_offset}");
            result.push(&text[offset..next_offset]);
            offset = next_offset + 1;
        } else {
            // println!("offset={offset} next_offset is none");
            result.push(&text[offset..]);
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_lines() {
        let lines = split_lines(b"foo\nbar\nbaz");
        assert_eq!(lines, vec![b"foo", b"bar", b"baz"]);
    }

    #[test]
    fn test_naive_matcher_tiny() {
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
