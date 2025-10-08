use eurorust_2025_workshop::simd_brightness::{
    brightness_autovec, brightness_scalar, brightness_simd,
};
use image::RgbImage;

fn main() {
    divan::main();
}

fn load_test_image() -> RgbImage {
    image::open("data/large.jpg")
        .expect("Failed to load test image.")
        .to_rgb8()
}

#[divan::bench(sample_count = 3, sample_size = 5)]
fn bench_brightness_scalar(bencher: divan::Bencher) {
    let img = load_test_image();

    bencher.bench(|| brightness_scalar(divan::black_box(&img), divan::black_box(30)));
}

#[divan::bench(sample_count = 3, sample_size = 5)]
fn bench_brightness_autovec(bencher: divan::Bencher) {
    let img = load_test_image();

    bencher.bench(|| brightness_autovec(divan::black_box(&img), divan::black_box(30)));
}

#[divan::bench(sample_count = 3, sample_size = 5)]
fn bench_brightness_simd(bencher: divan::Bencher) {
    let img = load_test_image();

    bencher.bench(|| brightness_simd(divan::black_box(&img), divan::black_box(30)));
}
