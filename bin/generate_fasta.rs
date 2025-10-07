use std::fs::File;
use std::io::{BufWriter, Write};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

fn main() -> std::io::Result<()> {
    // Use a fixed seed to ensure reproducibility
    let mut rng = StdRng::seed_from_u64(42);
    let file = File::create("genome.fasta")?;
    let mut writer = BufWriter::new(file);

    const TARGET_SIZE: usize = 200 * 1024 * 1024; // 200MB
    const SEQUENCE_LENGTH: usize = 80; // Standard FASTA line length
    const NUCLEOTIDES: &[u8] = b"ACGT";

    let mut current_size = 0;
    let mut sequence_id = 1;

    // Inject the target pattern in some sequences
    let pattern = b"AGTCCGTA";

    while current_size < TARGET_SIZE {
        // Write header
        let header = format!(">sequence_{}\n", sequence_id);
        writer.write_all(header.as_bytes())?;
        current_size += header.len();

        // Generate random sequence (around 1000 bases per sequence)
        let num_lines = rng.gen_range(10..15);
        for line_num in 0..num_lines {
            let mut line = Vec::with_capacity(SEQUENCE_LENGTH);

            // Occasionally inject the pattern
            if sequence_id % 100 == 0 && line_num == num_lines / 2 {
                line.extend_from_slice(pattern);
            }

            // Fill the rest with random nucleotides
            while line.len() < SEQUENCE_LENGTH {
                let nucleotide = NUCLEOTIDES[rng.gen_range(0..4)];
                line.push(nucleotide);
            }

            writer.write_all(&line)?;
            writer.write_all(b"\n")?;
            current_size += SEQUENCE_LENGTH + 1;

            if current_size >= TARGET_SIZE {
                break;
            }
        }

        sequence_id += 1;
    }

    writer.flush()?;
    println!("Generated genome.fasta (~{}MB)", current_size / (1024 * 1024));
    Ok(())
}
