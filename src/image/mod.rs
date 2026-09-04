pub mod spatial_predictor;

pub use spatial_predictor::{
    compress_image_grayscale, decompress_image_grayscale,
    compress_image_rgb, decompress_image_rgb,
    compress_image, decompress_image,
    filter_row, unfilter_row, select_best_filter, FilterType,
};
