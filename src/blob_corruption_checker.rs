use memmap2::Mmap;
use rayon::prelude::*;
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
    // Memory-map both files for efficient parallel access
    let ref_file = File::open(reference_path).unwrap();
    let corrupt_file = File::open(corrupted_path).unwrap();

    let ref_mmap = unsafe { Mmap::map(&ref_file).unwrap() };
    let corrupt_mmap = unsafe { Mmap::map(&corrupt_file).unwrap() };

    let ref_data = &ref_mmap[..];
    let corrupt_data = &corrupt_mmap[..];

    // Use large work units (10MB) to minimize parallelization overhead
    let work_unit_size = chunk_size * 10240; // 10MB chunks

    // Process in parallel with large work units
    let all_corruptions: Vec<Vec<Corruption>> = ref_data
        .par_chunks(work_unit_size)
        .zip(corrupt_data.par_chunks(work_unit_size))
        .enumerate()
        .map(|(work_idx, (ref_work, corrupt_work))| {
            let base_offset = (work_idx * work_unit_size) as u64;
            let mut local_corruptions: Vec<Corruption> = Vec::new();

            // Process chunks sequentially within this work unit
            for (chunk_offset, (ref_chunk, corrupt_chunk)) in ref_work
                .chunks(chunk_size)
                .zip(corrupt_work.chunks(chunk_size))
                .enumerate()
            {
                if ref_chunk != corrupt_chunk {
                    let offset = base_offset + (chunk_offset * chunk_size) as u64;
                    let length = ref_chunk.len() as u64;

                    if let Some(last) = local_corruptions.last_mut() {
                        if last.offset + last.length == offset {
                            last.length += length;
                        } else {
                            local_corruptions.push(Corruption { offset, length });
                        }
                    } else {
                        local_corruptions.push(Corruption { offset, length });
                    }
                }
            }

            local_corruptions
        })
        .collect();

    // Merge results from all work units
    let mut corruptions: Vec<Corruption> = Vec::new();
    for local_corruptions in all_corruptions {
        for corruption in local_corruptions {
            if let Some(last) = corruptions.last_mut() {
                if last.offset + last.length == corruption.offset {
                    last.length += corruption.length;
                } else {
                    corruptions.push(corruption);
                }
            } else {
                corruptions.push(corruption);
            }
        }
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
