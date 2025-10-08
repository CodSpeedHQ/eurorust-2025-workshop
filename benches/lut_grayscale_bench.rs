use eurorust_2025_workshop::lut_grayscale::*;
use image::{RgbImage};

fn main() {
    divan::main();
}

fn load_test_image() -> RgbImage {
    image::open("data/large.jpg")
        .expect("Failed to load test image")
        .to_rgb8()
}

#[divan::bench(sample_count = 3, sample_size = 5)]
fn bench_rgb_to_gray_naive(bencher: divan::Bencher) {
    let img = load_test_image();

    bencher.bench(|| rgb_to_gray_naive(divan::black_box(&img)));
}

#[divan::bench(sample_count = 3, sample_size = 5)]
fn bench_rgb_to_gray_small_lut(bencher: divan::Bencher) {
    let img = load_test_image();
    let lut = GrayscaleLut::new();

    bencher.bench(|| rgb_to_gray_small_lut(divan::black_box(&img), divan::black_box(&lut)));
}

#[divan::bench(sample_count = 3, sample_size = 5)]
fn bench_rgb_to_gray_big_lut(bencher: divan::Bencher) {
    let img = load_test_image();
    let lut = GrayscaleLutBig::new();

    bencher.bench(|| rgb_to_gray_big_lut(divan::black_box(&img), divan::black_box(&lut)));
}
