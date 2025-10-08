use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Corruption {
    /// Offset is aligned to the chunk_size boundary (e.g., 1KB = 1024 bytes)
    pub offset: u64,
    /// Length is a multiple of chunk_size
    pub length: u64,
}

pub fn find_corruptions_sequential(
    reference_path: &str,
    corrupted_path: &str,
    chunk_size: usize,
) -> Vec<Corruption> {
    let mut ref_file = BufReader::with_capacity(1024 * 1024, File::open(reference_path).unwrap());
    let mut corrupt_file =
        BufReader::with_capacity(1024 * 1024, File::open(corrupted_path).unwrap());

    let mut ref_buffer = vec![0u8; chunk_size];
    let mut corrupt_buffer = vec![0u8; chunk_size];

    let mut corruptions: Vec<Corruption> = Vec::new();
    let mut offset = 0u64;

    loop {
        let n = ref_file.read(&mut ref_buffer).unwrap();
        if n == 0 {
            break;
        }

        corrupt_file.read_exact(&mut corrupt_buffer[..n]).unwrap();

        // Compare byte by byte and track consecutive corrupted chunks
        if ref_buffer[..n] != corrupt_buffer[..n] {
            // Check if this continues the previous corruption
            if let Some(last) = corruptions.last_mut() {
                if last.offset + last.length == offset {
                    // Extend the previous corruption
                    last.length += n as u64;
                } else {
                    // New corruption
                    corruptions.push(Corruption {
                        offset,
                        length: n as u64,
                    });
                }
            } else {
                // First corruption
                corruptions.push(Corruption {
                    offset,
                    length: n as u64,
                });
            }
        }

        offset += n as u64;
    }

    corruptions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_corruptions_sequential() {
        let corruptions = find_corruptions_sequential("reference.bin", "corrupted.bin", 1024);

        assert_eq!(corruptions.len(), 50, "Should find 50 corruptions");

        // All corruptions should be 1KB aligned
        for corruption in &corruptions {
            assert_eq!(
                corruption.offset % 1024,
                0,
                "Corruption offset should be 1KB aligned"
            );
            assert_eq!(
                corruption.length % 1024,
                0,
                "Corruption length should be multiple of 1KB"
            );
        }

        // Check specific corruptions
        assert_eq!(corruptions[0].offset, 14801920, "First corruption offset");
        assert_eq!(corruptions[0].length, 2048, "First corruption length");
        assert_eq!(
            corruptions[25].offset, 243891200,
            "Middle corruption offset"
        );
        assert_eq!(corruptions[25].length, 4096, "Middle corruption length");
        assert_eq!(corruptions[49].offset, 507871232, "Last corruption offset");
        assert_eq!(corruptions[49].length, 5120, "Last corruption length");
    }
}
