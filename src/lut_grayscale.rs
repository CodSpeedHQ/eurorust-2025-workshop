/// Lookup Table (LUT) Challenge: RGB to Grayscale Conversion
///
/// In this challenge, you'll implement an optimized grayscale conversion using a lookup table.
///
/// ## The Problem
/// Converting RGB to grayscale using the standard luminosity formula:
/// Gray = 0.299 * R + 0.587 * G + 0.114 * B
///
/// This requires 5 floating-point operations per pixel (3 multiplications + 2 additions).
/// With millions of pixels, this adds up quickly!
///
/// ## The Solution: Lookup Tables
/// Since RGB values are 0-255, we can pre-compute results and store them in arrays.
/// This trades computation for memory access.
use image::{GrayImage, ImageBuffer, Luma, RgbImage};

/// Pre-computed lookup tables for each RGB channel
/// Memory: 768 bytes (3 * 256)
pub struct GrayscaleLut {
    red_lut: [u8; 256],
    green_lut: [u8; 256],
    blue_lut: [u8; 256],
}

impl GrayscaleLut {
    /// Create a new lookup table with standard luminosity weights
    pub fn new() -> Self {
        let mut red_lut = [0u8; 256];
        let mut green_lut = [0u8; 256];
        let mut blue_lut = [0u8; 256];

        for i in 0..256 {
            red_lut[i] = (i as f32 * 0.299) as u8;
            green_lut[i] = (i as f32 * 0.587) as u8;
            blue_lut[i] = (i as f32 * 0.114) as u8;
        }

        Self {
            red_lut,
            green_lut,
            blue_lut,
        }
    }
}

impl Default for GrayscaleLut {
    fn default() -> Self {
        Self::new()
    }
}

/// Giant 3D lookup table with ALL RGB combinations pre-computed
/// Memory: 16,777,216 bytes (~16 MB) for 256^3 entries
///
/// This is the ultimate space-time tradeoff:
/// - Uses 16MB of memory
/// - Single array lookup per pixel (no additions!)
/// - May not fit in CPU cache
pub struct GrayscaleLutBig {
    lut: Box<[[[u8; 256]; 256]; 256]>,
}

impl GrayscaleLutBig {
    pub fn new() -> Self {
        // Allocate directly on heap to avoid stack overflow
        let mut lut = vec![0u8; 256 * 256 * 256].into_boxed_slice();

        for r in 0..256 {
            for g in 0..256 {
                for b in 0..256 {
                    let gray = (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) as u8;
                    let idx = r * 256 * 256 + g * 256 + b;
                    lut[idx] = gray;
                }
            }
        }

        // Convert Box<[u8]> to the proper array type
        let lut_ptr = Box::into_raw(lut) as *mut [[[u8; 256]; 256]; 256];
        let lut = unsafe { Box::from_raw(lut_ptr) };

        Self { lut }
    }
}

impl Default for GrayscaleLutBig {
    fn default() -> Self {
        Self::new()
    }
}

/// Naive implementation: computes grayscale using floating-point math for every pixel
///
/// This is SLOW because:
/// 1. 5 floating-point operations per pixel (3 multiplications + 2 additions)
/// 2. Type conversions (u8 -> f32 -> u8)
/// 3. No pre-computation
pub fn rgb_to_gray_naive(img: &RgbImage) -> GrayImage {
    let (width, height) = img.dimensions();
    let mut gray_img = ImageBuffer::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels() {
        let r = pixel[0] as f32;
        let g = pixel[1] as f32;
        let b = pixel[2] as f32;

        // Compute grayscale value using luminosity formula
        let gray = (r * 0.299 + g * 0.587 + b * 0.114) as u8;

        gray_img.put_pixel(x, y, Luma([gray]));
    }

    gray_img
}

/// Optimized implementation using separate lookup tables
///
/// This should be MUCH faster than the naive version because:
/// 1. No floating-point operations per pixel
/// 2. Only 3 array lookups + 2 integer additions
/// 3. Better CPU cache locality (768 bytes fits in L1 cache)
pub fn rgb_to_gray_small_lut(img: &RgbImage, lut: &GrayscaleLut) -> GrayImage {
    let (width, height) = img.dimensions();
    let mut gray_img = ImageBuffer::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels() {
        let r = pixel[0] as usize;
        let g = pixel[1] as usize;
        let b = pixel[2] as usize;

        // Use lookup tables instead of computing
        let gray_value = lut.red_lut[r]
            .saturating_add(lut.green_lut[g])
            .saturating_add(lut.blue_lut[b]);

        gray_img.put_pixel(x, y, Luma([gray_value]));
    }

    gray_img
}

/// Optimized implementation using a big lookup table
///
/// Trade-offs:
/// - Memory: 16 MB (likely exceeds L1/L2 cache)
/// - Computation: Single array access, NO additions
/// - Question: Is this actually faster? Cache misses might hurt!
pub fn rgb_to_gray_big_lut(img: &RgbImage, lut: &GrayscaleLutBig) -> GrayImage {
    let (width, height) = img.dimensions();
    let mut gray_img = ImageBuffer::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels() {
        let r = pixel[0] as usize;
        let g = pixel[1] as usize;
        let b = pixel[2] as usize;

        // Single lookup - all values pre-computed!
        let gray_value = lut.lut[r][g][b];

        gray_img.put_pixel(x, y, Luma([gray_value]));
    }

    gray_img
}

#[cfg(test)]
mod tests {
    use crate::helpers::assert_eq_gray_img;

    use super::*;
    use image::Rgb;

    fn test_impl(func: fn(&RgbImage) -> GrayImage) {
        let img = ImageBuffer::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                Rgb([255u8, 0, 0]) // Red
            } else {
                Rgb([0u8, 255, 0]) // Green
            }
        });
        let gray = func(&img);

        assert_eq!(gray.dimensions(), (2, 2));
        assert_eq!(gray.get_pixel(0, 0)[0], 76); // Red -> 76
        assert_eq!(gray.get_pixel(1, 0)[0], 149); // Green -> 149
    }

    #[test]
    fn test_with_real_image() {
        let img = image::open("data/small.jpg").unwrap().to_rgb8();
        let naive = rgb_to_gray_naive(&img);

        let lut = GrayscaleLut::new();
        let small_lut = rgb_to_gray_small_lut(&img, &lut);
        let big_lut = GrayscaleLutBig::new();
        let big_lut = rgb_to_gray_big_lut(&img, &big_lut);

        assert_eq_gray_img(&naive, &small_lut);
        assert_eq_gray_img(&naive, &big_lut);

        naive.save("test_grayscale_naive.png").unwrap();
        small_lut.save("test_grayscale_small_lut.png").unwrap();
        big_lut.save("test_grayscale_big_lut.png").unwrap();
    }

    #[test]
    fn test_rgb_to_gray_naive() {
        test_impl(rgb_to_gray_naive);
    }

    #[test]
    fn test_rgb_to_gray_small_lut() {
        test_impl(|img| {
            let lut = GrayscaleLut::new();
            rgb_to_gray_small_lut(img, &lut)
        });
    }

    #[test]
    fn test_rgb_to_gray_big_lut() {
        test_impl(|img| {
            let lut = GrayscaleLutBig::new();
            rgb_to_gray_big_lut(img, &lut)
        });
    }
}
