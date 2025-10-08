use image::{GrayImage, RgbImage};

pub fn assert_eq_img(img_1: &RgbImage, img_2: &RgbImage) {
    let result = image_compare::rgb_similarity_structure(
        &image_compare::Algorithm::RootMeanSquared,
        &img_1,
        &img_2,
    )
    .unwrap();
    assert!(result.score > 0.99);
}

pub fn assert_eq_gray_img(img_1: &GrayImage, img_2: &GrayImage) {
    let result = image_compare::gray_similarity_structure(
        &image_compare::Algorithm::RootMeanSquared,
        &img_1,
        &img_2,
    )
    .unwrap();
    assert!(result.score > 0.99);
}
