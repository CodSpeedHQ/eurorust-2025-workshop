use eurorust_2025_workshop::simd_filters::*;
use image::RgbImage;

fn main() {
    divan::main();
}

fn load_test_image() -> RgbImage {
    image::open("data/large.jpg")
        .expect("Failed to load test image")
        .to_rgb8()
}

#[divan::bench(sample_count = 2, sample_size = 3)]
fn bench_brightness_contrast(bencher: divan::Bencher) {
    let img = load_test_image();

    bencher.bench(|| {
        apply_brightness_contrast(
            divan::black_box(&img),
            divan::black_box(30),
            divan::black_box(0.3),
        )
    });
}

#[divan::bench(sample_count = 2, sample_size = 3)]
fn bench_gamma(bencher: divan::Bencher) {
    let img = load_test_image();

    bencher.bench(|| apply_gamma(divan::black_box(&img), divan::black_box(2.2)));
}

#[divan::bench(sample_count = 2, sample_size = 3)]
fn bench_brightness_contrast_gamma(bencher: divan::Bencher) {
    let img = load_test_image();

    bencher.bench(|| {
        apply_brightness_contrast_gamma(
            divan::black_box(&img),
            divan::black_box(30),
            divan::black_box(0.3),
            divan::black_box(2.2),
        )
    });
}
