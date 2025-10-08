use rand::{Rng, SeedableRng};
use std::fs::File;
use std::io::Write;

fn main() {
    const SIZE_MB: usize = 500; // File size in MB

    println!("Generating blob test files ({} MB)...", SIZE_MB);

    generate_blob("reference.bin", SIZE_MB, &[]).expect("Failed to generate reference.bin");

    // Generate random corruptions that scale with file size
    // 1 corruption per 10MB
    let num_corruptions = SIZE_MB / 10;
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut corruption_points = Vec::new();

    for _ in 0..num_corruptions {
        let offset = rng.gen_range(0..(SIZE_MB * 1024 * 1024) as u64);
        let length = rng.gen_range(512..4096); // Random length between 512 bytes and 4KB
        corruption_points.push((offset, length));
    }

    generate_blob("corrupted.bin", SIZE_MB, &corruption_points)
        .expect("Failed to generate corrupted.bin");

    println!("Done! Generated reference.bin and corrupted.bin");
}

/// Generate a blob file with the given size and optional corruption points
fn generate_blob(
    path: &str,
    size_mb: usize,
    corruption_points: &[(u64, u64)], // (offset, length) pairs to corrupt
) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    let size_bytes = size_mb * 1024 * 1024;

    // Generate deterministic data
    let chunk_size = 1024 * 1024; // 1MB chunks
    let mut buffer = vec![0u8; chunk_size];

    let mut written = 0;
    let mut chunk_id = 0u64;

    while written < size_bytes {
        let to_write = std::cmp::min(chunk_size, size_bytes - written);

        // Fill buffer with deterministic pattern
        for (i, byte) in buffer[..to_write].iter_mut().enumerate() {
            *byte = ((chunk_id + i as u64) % 256) as u8;
        }

        // Apply corruptions if any fall in this chunk
        for &(corrupt_offset, corrupt_length) in corruption_points {
            let chunk_start = written as u64;
            let chunk_end = chunk_start + to_write as u64;

            if corrupt_offset < chunk_end && corrupt_offset + corrupt_length > chunk_start {
                // This corruption overlaps with current chunk
                let local_start = corrupt_offset.saturating_sub(chunk_start) as usize;
                let local_end = std::cmp::min(
                    (corrupt_offset + corrupt_length - chunk_start) as usize,
                    to_write,
                );

                // Corrupt by XOR with 0xFF
                for byte in &mut buffer[local_start..local_end] {
                    *byte ^= 0xFF;
                }
            }
        }

        file.write_all(&buffer[..to_write])?;
        written += to_write;
        chunk_id += to_write as u64;
    }

    Ok(())
}
