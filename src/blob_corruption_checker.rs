use memmap2::Mmap;
use std::fs::File;

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
    // Memory-map both files for efficient sequential access
    let ref_file = File::open(reference_path).unwrap();
    let corrupt_file = File::open(corrupted_path).unwrap();

    let ref_mmap = unsafe { Mmap::map(&ref_file).unwrap() };
    let corrupt_mmap = unsafe { Mmap::map(&corrupt_file).unwrap() };

    let ref_data = &ref_mmap[..];
    let corrupt_data = &corrupt_mmap[..];

    let mut corruptions: Vec<Corruption> = Vec::new();
    let mut offset = 0u64;

    // Process chunks sequentially
    for (ref_chunk, corrupt_chunk) in ref_data.chunks(chunk_size).zip(corrupt_data.chunks(chunk_size)) {
        if ref_chunk != corrupt_chunk {
            if let Some(last) = corruptions.last_mut() {
                if last.offset + last.length == offset {
                    // Extend the previous corruption
                    last.length += ref_chunk.len() as u64;
                } else {
                    // New corruption
                    corruptions.push(Corruption {
                        offset,
                        length: ref_chunk.len() as u64,
                    });
                }
            } else {
                // First corruption
                corruptions.push(Corruption {
                    offset,
                    length: ref_chunk.len() as u64,
                });
            }
        }

        offset += ref_chunk.len() as u64;
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
        assert_eq!(
            corruptions[49].offset, 507871232,
            "Last corruption offset"
        );
        assert_eq!(corruptions[49].length, 5120, "Last corruption length");
    }
}
