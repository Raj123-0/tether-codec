use tether::image::{compress_image_grayscale, decompress_image_grayscale};

#[test]
fn test_image_gradient_roundtrip() {
    let width = 128u32;
    let height = 128u32;
    let mut pixels = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            let val = ((x * 3 + y * 2) % 256) as u8;
            pixels.push(val);
        }
    }

    let compressed = compress_image_grayscale(width, height, &pixels).unwrap();
    let raw_len = pixels.len();
    let comp_len = compressed.len();
    println!("Gradient image (128x128): raw {} B -> compressed {} B ({:.2}x)",
             raw_len, comp_len, raw_len as f64 / comp_len as f64);
    assert!(comp_len < raw_len / 2);

    let (dec_w, dec_h, dec_pixels) = decompress_image_grayscale(&compressed).unwrap();
    assert_eq!(dec_w, width);
    assert_eq!(dec_h, height);
    assert_eq!(dec_pixels, pixels);
}

#[test]
fn test_image_flat_ui_blocks() {
    let width = 200u32;
    let height = 100u32;
    let mut pixels = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            // Flat UI rectangular regions
            let val = if x < 50 && y < 50 {
                255
            } else if x >= 100 && y >= 50 {
                40
            } else if y % 20 == 0 {
                128
            } else {
                20
            };
            pixels.push(val);
        }
    }

    let compressed = compress_image_grayscale(width, height, &pixels).unwrap();
    let raw_len = pixels.len();
    let comp_len = compressed.len();
    println!("Flat UI screenshot (200x100): raw {} B -> compressed {} B ({:.2}x)",
             raw_len, comp_len, raw_len as f64 / comp_len as f64);
    assert!(comp_len < raw_len / 3);

    let (dec_w, dec_h, dec_pixels) = decompress_image_grayscale(&compressed).unwrap();
    assert_eq!(dec_w, width);
    assert_eq!(dec_h, height);
    assert_eq!(dec_pixels, pixels);
}

#[test]
fn test_image_edge_dimensions() {
    let dims = [(1, 1), (2, 2), (1, 50), (50, 1), (7, 13)];
    for (w, h) in dims {
        let pixels: Vec<u8> = (0..(w * h)).map(|i| (i * 17 % 256) as u8).collect();
        let compressed = compress_image_grayscale(w, h, &pixels).unwrap();
        let (dec_w, dec_h, dec_pixels) = decompress_image_grayscale(&compressed).unwrap();
        assert_eq!(dec_w, w);
        assert_eq!(dec_h, h);
        assert_eq!(dec_pixels, pixels);
    }
}
