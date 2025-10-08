/// Multi-Filter LUT Challenge: Apply multiple image filters using lookup tables
///
/// ## Your Task
/// Implement optimized versions of these filters using lookup tables:
/// 1. Brightness/Contrast adjustment
/// 2. Gamma correction
/// 3. Combined (optional)
///
/// ## Why LUTs?
/// - Pre-compute expensive operations (like powf for gamma)
/// - Transform floating-point math into simple array lookups
/// - Process millions of pixels with minimal computation
///
/// ## Learning Goals
/// - Understand space-time tradeoffs
/// - See dramatic speedups (especially for gamma: 50-100x!)
/// - Learn when LUTs are appropriate
use image::{ImageBuffer, Rgb, RgbImage};

pub fn apply_brightness_contrast(img: &RgbImage, brightness: i16, contrast: f32) -> RgbImage {
    naive::apply_brightness_contrast(img, brightness, contrast)
}

pub fn apply_gamma(img: &RgbImage, gamma: f32) -> RgbImage {
    naive::apply_gamma(img, gamma)
}

pub fn apply_brightness_contrast_gamma(
    img: &RgbImage,
    brightness: i16,
    contrast: f32,
    gamma: f32,
) -> RgbImage {
    let temp_img = apply_brightness_contrast(img, brightness, contrast);
    naive::apply_gamma(&temp_img, gamma)
}

mod naive {
    use super::*;

    /// Apply brightness and contrast with floating-point math per pixel
    pub fn apply_brightness_contrast(img: &RgbImage, brightness: i16, contrast: f32) -> RgbImage {
        let (width, height) = img.dimensions();
        let mut output = ImageBuffer::new(width, height);
        let mut c_table: [u8; 256] = [0; 256];
        

        for i in 0..256 {
            let f = i as f32;
            c_table[i] = (((f - 128.0) * (1.0 + contrast)) + 128.0 + (brightness as f32)).clamp(0.0, 255.0) as u8;
        }



        for (x, y, pixel) in img.enumerate_pixels() {
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            output.put_pixel(
                x,
                y,
                Rgb([
                    c_table[r as usize],
                    c_table[g as usize],
                    c_table[b as usize],
                ]),
            );
        }

        output
    }

    /// Naive implementation: Apply gamma correction
    /// This is VERY slow because powf() is expensive!
    pub fn apply_gamma(img: &RgbImage, gamma: f32) -> RgbImage {
        let (width, height) = img.dimensions();
        let mut output = ImageBuffer::new(width, height);

        for (x, y, pixel) in img.enumerate_pixels() {
            // powf() is VERY expensive - this is why we need a LUT!
            let r = (pixel[0] as f32 / 255.0).powf(1.0 / gamma) * 255.0;
            let g = (pixel[1] as f32 / 255.0).powf(1.0 / gamma) * 255.0;
            let b = (pixel[2] as f32 / 255.0).powf(1.0 / gamma) * 255.0;

            output.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn create_test_image() -> RgbImage {
        ImageBuffer::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                Rgb([128u8, 128, 128]) // Gray
            } else {
                Rgb([200u8, 100, 50]) // Mixed colors
            }
        })
    }

    fn hash_image(img: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> u64 {
        let mut hasher = DefaultHasher::new();
        img.as_raw().hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_with_real_image() {
        let img = image::open("data/small.jpg").unwrap().to_rgb8();
        let brightness_contrast = apply_brightness_contrast(&img, 40, 0.2);
        let gamma = apply_gamma(&img, 2.2);
        let all = apply_brightness_contrast_gamma(&img, 40, 0.2, 2.2);

        brightness_contrast
            .save("test_lut_filters_brightness_contrast.png")
            .unwrap();
        gamma.save("test_lut_filters_gamma.png").unwrap();
        all.save("test_lut_filters.png").unwrap();
    }

    #[test]
    fn test_apply_brightness_contrast() {
        let img = create_test_image();
        let result = apply_brightness_contrast(&img, 20, 0.5);

        assert_eq!(result.dimensions(), (2, 2));
        assert_eq!(hash_image(&result), 18018001747868405400);
    }

    #[test]
    fn test_apply_gamma() {
        let img = create_test_image();
        let result = apply_gamma(&img, 2.2);

        assert_eq!(result.dimensions(), (2, 2));
        assert_eq!(hash_image(&result), 8273371144845572421);
    }

    #[test]
    fn test_apply_brightness_contrast_gamma() {
        let img = create_test_image();
        let result = apply_brightness_contrast_gamma(&img, 20, 0.5, 2.2);

        assert_eq!(result.dimensions(), (2, 2));
        assert_eq!(hash_image(&result), 14732392309420196016);
    }

    #[test]
    fn test_brightness_contrast_extreme_values() {
        let img = create_test_image();

        // Test high brightness (100)
        let result = apply_brightness_contrast(&img, 100, 0.0);
        assert_eq!(hash_image(&result), 13452353979130985766);

        // Test high contrast (2.0)
        let result = apply_brightness_contrast(&img, 0, 2.0);
        assert_eq!(hash_image(&result), 13900789802552529386);

        // Test negative brightness (-50)
        let result = apply_brightness_contrast(&img, -50, 0.0);
        assert_eq!(hash_image(&result), 9063327795097964491);
    }

    #[test]
    fn test_gamma_extreme_values() {
        let img = create_test_image();

        // Test low gamma (0.5 - brightening)
        let result = apply_gamma(&img, 0.5);
        assert_eq!(hash_image(&result), 15720564692747358114);

        // Test high gamma (3.0 - darkening)
        let result = apply_gamma(&img, 3.0);
        assert_eq!(hash_image(&result), 15646045841196030320);
    }
}
