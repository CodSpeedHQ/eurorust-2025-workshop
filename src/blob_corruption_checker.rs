use std::fs::File;
use std::io::{BufReader, Read};
use std::simd::{prelude::*, LaneCount, SupportedLaneCount};
use memmap2::Mmap;
use rayon::prelude::*;

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
    let mut ref_file = BufReader::new(File::open(reference_path).unwrap());
    let mut corrupt_file = BufReader::new(File::open(corrupted_path).unwrap());

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

pub fn find_corruptions_parallel(
    reference_path: &str,
    corrupted_path: &str,
    chunk_size: usize,
) -> Vec<Corruption> {
    // Memory-map both files for fast access
    let ref_file = File::open(reference_path).unwrap();
    let corrupt_file = File::open(corrupted_path).unwrap();

    let ref_mmap = unsafe { Mmap::map(&ref_file).unwrap() };
    let corrupt_mmap = unsafe { Mmap::map(&corrupt_file).unwrap() };

    let file_len = ref_mmap.len();

    // Process chunks in parallel
    let mut chunk_mismatches: Vec<(usize, bool)> = (0..file_len)
        .step_by(chunk_size)
        .collect::<Vec<_>>()
        .par_iter()
        .map(|&offset| {
            let end = (offset + chunk_size).min(file_len);
            let matches = ref_mmap[offset..end] == corrupt_mmap[offset..end];
            (offset, !matches)
        })
        .collect();

    // Sort by offset to ensure correct ordering
    chunk_mismatches.sort_by_key(|(offset, _)| *offset);

    // Merge consecutive corrupted chunks
    let mut corruptions = Vec::new();
    let mut current_corruption: Option<Corruption> = None;

    for (offset, is_corrupted) in chunk_mismatches {
        if is_corrupted {
            let chunk_len = ((offset + chunk_size).min(file_len) - offset) as u64;

            if let Some(mut corruption) = current_corruption.take() {
                if corruption.offset + corruption.length == offset as u64 {
                    // Extend existing corruption
                    corruption.length += chunk_len;
                    current_corruption = Some(corruption);
                } else {
                    // Push previous and start new
                    corruptions.push(corruption.clone());
                    current_corruption = Some(Corruption {
                        offset: offset as u64,
                        length: chunk_len,
                    });
                }
            } else {
                // Start first corruption
                current_corruption = Some(Corruption {
                    offset: offset as u64,
                    length: chunk_len,
                });
            }
        }
    }

    // Don't forget the last corruption
    if let Some(corruption) = current_corruption {
        corruptions.push(corruption);
    }

    corruptions
}

/// SIMD-accelerated chunk comparison
fn chunks_equal_simd<const LANES: usize>(a: &[u8], b: &[u8]) -> bool
where
    LaneCount<LANES>: SupportedLaneCount,
{
    if a.len() != b.len() {
        return false;
    }

    let len = a.len();
    let simd_len = len - (len % LANES);

    // Process SIMD chunks
    let mut offset = 0;
    while offset < simd_len {
        let a_simd = Simd::<u8, LANES>::from_slice(&a[offset..offset + LANES]);
        let b_simd = Simd::<u8, LANES>::from_slice(&b[offset..offset + LANES]);

        if a_simd.simd_ne(b_simd).any() {
            return false;
        }
        offset += LANES;
    }

    // Handle remaining bytes
    a[offset..] == b[offset..]
}

pub fn find_corruptions_simd(
    reference_path: &str,
    corrupted_path: &str,
    chunk_size: usize,
) -> Vec<Corruption> {
    // Memory-map both files for fast access
    let ref_file = File::open(reference_path).unwrap();
    let corrupt_file = File::open(corrupted_path).unwrap();

    let ref_mmap = unsafe { Mmap::map(&ref_file).unwrap() };
    let corrupt_mmap = unsafe { Mmap::map(&corrupt_file).unwrap() };

    let file_len = ref_mmap.len();

    // Process chunks in parallel with SIMD comparisons
    let mut chunk_mismatches: Vec<(usize, bool)> = (0..file_len)
        .step_by(chunk_size)
        .map(|offset| {
            let end = (offset + chunk_size).min(file_len);
            let matches = chunks_equal_simd::<64>(&ref_mmap[offset..end], &corrupt_mmap[offset..end]);
            (offset, !matches)
        })
        .collect();

    // Sort by offset to ensure correct ordering
    chunk_mismatches.sort_by_key(|(offset, _)| *offset);

    // Merge consecutive corrupted chunks
    let mut corruptions = Vec::new();
    let mut current_corruption: Option<Corruption> = None;

    for (offset, is_corrupted) in chunk_mismatches {
        if is_corrupted {
            let chunk_len = ((offset + chunk_size).min(file_len) - offset) as u64;

            if let Some(ref mut corruption) = current_corruption {
                if corruption.offset + corruption.length == offset as u64 {
                    // Extend existing corruption
                    corruption.length += chunk_len;
                } else {
                    // Push previous and start new
                    corruptions.push(corruption.clone());
                    current_corruption = Some(Corruption {
                        offset: offset as u64,
                        length: chunk_len,
                    });
                }
            } else {
                // Start first corruption
                current_corruption = Some(Corruption {
                    offset: offset as u64,
                    length: chunk_len,
                });
            }
        }
    }

    // Don't forget the last corruption
    if let Some(corruption) = current_corruption {
        corruptions.push(corruption);
    }

    corruptions
}

pub fn find_corruptions_simd_parallel(
    reference_path: &str,
    corrupted_path: &str,
    chunk_size: usize,
) -> Vec<Corruption> {
    // Memory-map both files for fast access
    let ref_file = File::open(reference_path).unwrap();
    let corrupt_file = File::open(corrupted_path).unwrap();

    let ref_mmap = unsafe { Mmap::map(&ref_file).unwrap() };
    let corrupt_mmap = unsafe { Mmap::map(&corrupt_file).unwrap() };

    let file_len = ref_mmap.len();

    // Process chunks in parallel with SIMD comparisons
    let mut chunk_mismatches: Vec<(usize, bool)> = (0..file_len)
        .step_by(chunk_size)
        .collect::<Vec<_>>()
        .par_iter()
        .map(|&offset| {
            let end = (offset + chunk_size).min(file_len);
            let matches = chunks_equal_simd::<64>(&ref_mmap[offset..end], &corrupt_mmap[offset..end]);
            (offset, !matches)
        })
        .collect();

    // Sort by offset to ensure correct ordering
    chunk_mismatches.sort_by_key(|(offset, _)| *offset);

    // Merge consecutive corrupted chunks
    let mut corruptions = Vec::new();
    let mut current_corruption: Option<Corruption> = None;

    for (offset, is_corrupted) in chunk_mismatches {
        if is_corrupted {
            let chunk_len = ((offset + chunk_size).min(file_len) - offset) as u64;

            if let Some(ref mut corruption) = current_corruption {
                if corruption.offset + corruption.length == offset as u64 {
                    // Extend existing corruption
                    corruption.length += chunk_len;
                } else {
                    // Push previous and start new
                    corruptions.push(corruption.clone());
                    current_corruption = Some(Corruption {
                        offset: offset as u64,
                        length: chunk_len,
                    });
                }
            } else {
                // Start first corruption
                current_corruption = Some(Corruption {
                    offset: offset as u64,
                    length: chunk_len,
                });
            }
        }
    }

    // Don't forget the last corruption
    if let Some(corruption) = current_corruption {
        corruptions.push(corruption);
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

    #[test]
    fn test_find_corruptions_parallel() {
        let corruptions = find_corruptions_parallel("reference.bin", "corrupted.bin", 1024);

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

    #[test]
    fn test_find_corruptions_simd() {
        let corruptions = find_corruptions_simd("reference.bin", "corrupted.bin", 1024);

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

    #[test]
    fn test_find_corruptions_simd_parallel() {
        let corruptions = find_corruptions_simd_parallel("reference.bin", "corrupted.bin", 1024);

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
