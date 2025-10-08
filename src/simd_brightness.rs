use std::simd::{
    cmp::SimdOrd,
    num::{SimdInt, SimdUint},
};

/// SIMD Challenge: Process multiple pixels simultaneously
///
/// SIMD (Single Instruction, Multiple Data) allows processing multiple values
/// with a single CPU instruction. Modern CPUs have wide SIMD registers:
/// - SSE: 128-bit (16 bytes at once)
/// - AVX2: 256-bit (32 bytes at once)
/// - AVX-512: 512-bit (64 bytes at once)
///
/// This module demonstrates:
/// 1. Scalar (one-at-a-time) processing
/// 2. Auto-vectorization (compiler does SIMD automatically)
/// 3. Explicit SIMD using portable_simd
///
/// Perfect example: Brightness adjustment (add constant to each pixel)
use image::{ImageBuffer, Rgb, RgbImage};

/// Naive scalar implementation: Process one pixel at a time
pub fn brightness_scalar(img: &RgbImage, adjustment: i16) -> RgbImage {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels() {
        let r = (pixel[0] as i16 + adjustment).clamp(0, 255) as u8;
        let g = (pixel[1] as i16 + adjustment).clamp(0, 255) as u8;
        let b = (pixel[2] as i16 + adjustment).clamp(0, 255) as u8;

        output.put_pixel(x, y, Rgb([r, g, b]));
    }

    output
}

/// Auto-vectorized: Compiler-friendly code that LLVM can auto-vectorize
///
/// Key optimizations for auto-vectorization:
/// - Direct buffer access (no put_pixel calls)
/// - Simple loop structure
/// - No complex control flow
/// - Using saturating operations when possible
pub fn brightness_autovec(img: &RgbImage, adjustment: i16) -> RgbImage {
    let (width, height) = img.dimensions();

    let input = img.as_raw();
    let mut output = vec![0u8; input.len()];

    // Compiler can auto-vectorize this simple loop
    for i in 0..input.len() {
        let value = input[i] as i16 + adjustment;
        output[i] = value.clamp(0, 255) as u8;
    }

    ImageBuffer::from_raw(width, height, output).unwrap()
}

/// Explicit SIMD using std::simd (portable_simd)
///
/// This uses Rust's portable SIMD to explicitly process 16 bytes at once.
/// Benefits:
/// - Guaranteed SIMD (no relying on compiler)
/// - Can use specialized SIMD operations
/// - Portable across architectures
///
/// Note: Requires nightly Rust for now
pub fn brightness_simd(img: &RgbImage, adjustment: i16) -> RgbImage {
    use std::simd::{Simd, i16x16, u8x16};

    let (width, height) = img.dimensions();

    let input = img.as_raw();
    let mut output = vec![0u8; input.len()];

    let adjust_vec = Simd::splat(adjustment);

    // Process 16 bytes at a time
    let chunks = input.chunks_exact(16);
    let remainder = chunks.remainder();

    for (i, chunk) in chunks.enumerate() {
        // Load 16 u8 values
        let pixels = u8x16::from_slice(chunk);

        // Convert to i16 for safe arithmetic
        let pixels_i16: i16x16 = pixels.cast();

        // Add adjustment
        let adjusted = pixels_i16 + adjust_vec;

        // Clamp to 0-255 range
        let clamped = adjusted.simd_clamp(Simd::splat(0), Simd::splat(255));

        // Convert back to u8
        let result: u8x16 = clamped.cast();

        // Store result
        result.copy_to_slice(&mut output[i * 16..(i + 1) * 16]);
    }

    // Handle remaining bytes
    for (i, &byte) in remainder.iter().enumerate() {
        let value = byte as i16 + adjustment;
        output[input.len() - remainder.len() + i] = value.clamp(0, 255) as u8;
    }

    ImageBuffer::from_raw(width, height, output).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::helpers::assert_eq_img;

    use super::*;
    use image::Rgb;

    fn create_test_image() -> RgbImage {
        ImageBuffer::from_fn(4, 4, |x, y| Rgb([(x * 50) as u8, (y * 50) as u8, 128]))
    }

    #[test]
    fn test_with_real_image() {
        let img = image::open("data/small.jpg").unwrap().to_rgb8();
        let scalar = brightness_scalar(&img, 40);
        let autovec = brightness_autovec(&img, 40);
        let simd = brightness_simd(&img, 40);

        assert_eq_img(&scalar, &autovec);
        assert_eq_img(&scalar, &simd);

        scalar.save("test_simd_brightness_scalar.png").unwrap();
        autovec.save("test_simd_brightness_autovec.png").unwrap();
        simd.save("test_simd_brightness_simd.png").unwrap();
    }

    #[test]
    fn test_brightness_scalar() {
        let img = create_test_image();
        let result = brightness_scalar(&img, 20);

        assert_eq!(result.dimensions(), (4, 4));
        // Check that brightness was applied
        let pixel = result.get_pixel(0, 0);
        assert_eq!(pixel[2], 148); // 128 + 20
    }

    #[test]
    fn test_brightness_autovec() {
        let img = create_test_image();
        let result = brightness_autovec(&img, 20);

        assert_eq!(result.dimensions(), (4, 4));
        let pixel = result.get_pixel(0, 0);
        assert_eq!(pixel[2], 148);
    }

    #[test]
    fn test_brightness_clamping() {
        let img = create_test_image();
        let result = brightness_scalar(&img, 200);

        // Should clamp at 255
        let pixel = result.get_pixel(3, 3);
        assert_eq!(pixel[0], 255);
        assert_eq!(pixel[1], 255);
        assert_eq!(pixel[2], 255);
    }

    #[test]
    fn test_results_match() {
        let img = create_test_image();

        let scalar = brightness_scalar(&img, 30);
        let autovec = brightness_autovec(&img, 30);

        // Both should produce identical results
        assert_eq!(scalar.as_raw(), autovec.as_raw());
    }
}
