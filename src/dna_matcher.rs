use memchr::Memchr;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

struct ByteSplitImpl<'a> {
    iter: Memchr<'a>,
    slice: &'a [u8],
    position: usize,
    add_next: bool,
}

trait ByteSplit<'a> {
    fn byte_split(self, separator: u8) -> ByteSplitImpl<'a>;
}

impl<'a> ByteSplit<'a> for &'a [u8] {
    fn byte_split(self, separator: u8) -> ByteSplitImpl<'a> {
        ByteSplitImpl {
            iter: memchr::memchr_iter(separator, self),
            slice: self,
            position: 0,
            add_next: true,
        }
    }
}

impl<'a> Iterator for ByteSplitImpl<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_position) = self.iter.next() {
            let slice = self.slice.get(self.position..next_position);
            self.position = next_position + 1;
            self.add_next = true;
            return slice;
        }

        // If the iterator is consumed check if the last part of the string
        // is missing to be added.
        if !self.add_next {
            None
        } else {
            // Use case for reading from last comma to end.
            let slice = self.slice.get(self.position..);
            self.position = self.slice.len();
            self.add_next = false;
            slice
        }
    }
}

/// Naive approach: Read the entire file as a string and filter lines
pub fn naive_dna_matcher<'a>(genome: &'a [u8], pattern: &[u8]) -> Vec<&'a [u8]> {
    let matcher = jetscii::ByteSubstring::new(pattern);
    let lines = genome.byte_split(b'\n').collect::<Vec<_>>();
    lines
        .into_par_iter()
        .filter(|b| b.len() > 1 && b[0] != b'>' && matcher.find(b).is_some())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naive_matcher() {
        let test_genome = b">seq1\nACGTACGT\n>seq2\nAGTCCGTAAA\n>seq3\nGGGGGG";
        let pattern = b"AGTCCGTA";
        let matches = naive_dna_matcher(test_genome, pattern);
        println!("{:?}", matches);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], b"AGTCCGTAAA");
    }

    #[test]
    fn test_naive_matcher_on_genome_file() {
        // Read the actual genome.fasta file
        let genome = std::fs::read("genome.fasta")
            .expect("Failed to read genome.fasta\n\n Make sure to run 'cargo run --release --bin generate_fasta'");
        let pattern = b"AGTCCGTA";

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
            "✓ Found {} sequences containing pattern '{:?}'",
            matches.len(),
            pattern
        );
    }
}
