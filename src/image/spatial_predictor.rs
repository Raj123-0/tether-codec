// 2D Spatial causal predictors and row-by-row streaming image compression.

use crate::Vec;
use crate::entropy::{AdaptiveTable, RansEncoder, RansDecoder};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FilterType {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
    Plane = 5,
    Med = 6,
    RepeatPrev = 7,
    Constant = 8,
}

impl FilterType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::None),
            1 => Some(Self::Sub),
            2 => Some(Self::Up),
            3 => Some(Self::Average),
            4 => Some(Self::Paeth),
            5 => Some(Self::Plane),
            6 => Some(Self::Med),
            7 => Some(Self::RepeatPrev),
            8 => Some(Self::Constant),
            _ => None,
        }
    }
}

#[inline]
pub fn paeth_predict(a: u8, b: u8, c: u8) -> u8 {
    let a = a as i32;
    let b = b as i32;
    let c = c as i32;
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

#[inline]
pub fn med_predict(a: u8, b: u8, c: u8) -> u8 {
    let min_ab = a.min(b);
    let max_ab = a.max(b);
    if c >= max_ab {
        min_ab
    } else if c <= min_ab {
        max_ab
    } else {
        (a as i32 + b as i32 - c as i32) as u8
    }
}

#[inline]
pub fn plane_predict(a: u8, b: u8, c: u8) -> u8 {
    (a as i32 + b as i32 - c as i32) as u8
}

/// Filter a row of pixels given the previous row and chosen filter type.
pub fn filter_row(curr: &[u8], prev: &[u8], filter: FilterType, out: &mut Vec<u8>) {
    out.reserve(curr.len());
    let mut left = 0u8;
    for i in 0..curr.len() {
        let above = if !prev.is_empty() { prev[i] } else { 0 };
        let above_left = if i > 0 && !prev.is_empty() { prev[i - 1] } else { 0 };
        let pred = match filter {
            FilterType::None => 0,
            FilterType::Sub => left,
            FilterType::Up => above,
            FilterType::Average => ((left as u16 + above as u16) / 2) as u8,
            FilterType::Paeth => paeth_predict(left, above, above_left),
            FilterType::Plane => plane_predict(left, above, above_left),
            FilterType::Med => med_predict(left, above, above_left),
            _ => 0,
        };
        let res = curr[i].wrapping_sub(pred);
        out.push(res);
        left = curr[i];
    }
}

/// Invert filtering on a row of residuals to reconstruct pixel values.
pub fn unfilter_row(residuals: &[u8], prev: &[u8], filter: FilterType, out: &mut Vec<u8>) {
    out.reserve(residuals.len());
    let mut left = 0u8;
    for i in 0..residuals.len() {
        let above = if !prev.is_empty() { prev[i] } else { 0 };
        let above_left = if i > 0 && !prev.is_empty() { prev[i - 1] } else { 0 };
        let pred = match filter {
            FilterType::None => 0,
            FilterType::Sub => left,
            FilterType::Up => above,
            FilterType::Average => ((left as u16 + above as u16) / 2) as u8,
            FilterType::Paeth => paeth_predict(left, above, above_left),
            FilterType::Plane => plane_predict(left, above, above_left),
            FilterType::Med => med_predict(left, above, above_left),
            _ => 0,
        };
        let val = residuals[i].wrapping_add(pred);
        out.push(val);
        left = val;
    }
}

/// Adaptively select best filter type for a row by minimizing residual sum.
pub fn select_best_filter(curr: &[u8], prev: &[u8]) -> FilterType {
    let candidates = [
        FilterType::None,
        FilterType::Sub,
        FilterType::Up,
        FilterType::Average,
        FilterType::Paeth,
        FilterType::Plane,
        FilterType::Med,
    ];

    let mut best_filter = FilterType::None;
    let mut best_score = u32::MAX;

    let mut temp = Vec::with_capacity(curr.len());
    for &f in &candidates {
        temp.clear();
        filter_row(curr, prev, f, &mut temp);
        let score: u32 = temp.iter().map(|&r| {
            if r <= 128 { r as u32 } else { 256 - r as u32 }
        }).sum();

        if score < best_score {
            best_score = score;
            best_filter = f;
        }
    }

    best_filter
}

/// Encode a single 8-bit image channel row with RepeatPrev and Constant fast paths.
fn encode_channel_row(
    curr_row: &[u8],
    prev_row: &mut Vec<u8>,
    table: &mut AdaptiveTable,
    row_residuals: &mut Vec<u8>,
    out: &mut Vec<u8>,
) {
    if !prev_row.is_empty() && curr_row == prev_row.as_slice() {
        out.push(FilterType::RepeatPrev as u8);
        out.extend_from_slice(&0u32.to_le_bytes());
        return;
    }

    let first = curr_row[0];
    if curr_row.iter().all(|&x| x == first) {
        out.push(FilterType::Constant as u8);
        out.extend_from_slice(&1u32.to_le_bytes());
        out.push(first);
        prev_row.clear();
        prev_row.extend_from_slice(curr_row);
        return;
    }

    let filter = select_best_filter(curr_row, prev_row);
    row_residuals.clear();
    filter_row(curr_row, prev_row, filter, row_residuals);

    let mut rans = RansEncoder::new();
    rans.encode_block(row_residuals, table);
    let payload = rans.finish();

    for &r in row_residuals.iter() {
        table.observe(r);
    }

    out.push(filter as u8);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&payload);

    prev_row.clear();
    prev_row.extend_from_slice(curr_row);
}

/// Decode a single 8-bit image channel row.
fn decode_channel_row(
    w: usize,
    compressed: &[u8],
    offset: &mut usize,
    prev_row: &mut Vec<u8>,
    table: &mut AdaptiveTable,
    curr_row: &mut Vec<u8>,
) -> Result<(), &'static str> {
    if *offset + 5 > compressed.len() {
        return Err("Truncated image row header");
    }
    let filter_byte = compressed[*offset];
    *offset += 1;
    let payload_len = u32::from_le_bytes(compressed[*offset..*offset + 4].try_into().unwrap()) as usize;
    *offset += 4;

    let filter = FilterType::from_u8(filter_byte).ok_or("Invalid filter type")?;

    match filter {
        FilterType::RepeatPrev => {
            if prev_row.len() != w {
                return Err("RepeatPrev on first row or mismatched width");
            }
            curr_row.clear();
            curr_row.extend_from_slice(prev_row);
        }
        FilterType::Constant => {
            if *offset >= compressed.len() {
                return Err("Truncated constant row payload");
            }
            let val = compressed[*offset];
            *offset += payload_len;
            curr_row.clear();
            curr_row.extend(core::iter::repeat(val).take(w));
            prev_row.clear();
            prev_row.extend_from_slice(curr_row);
        }
        _ => {
            if *offset + payload_len > compressed.len() {
                return Err("Truncated image row payload");
            }
            let payload = &compressed[*offset..*offset + payload_len];
            *offset += payload_len;

            let mut rans = RansDecoder::new(payload)?;
            let residuals = rans.decode_block(w, table);

            for &r in &residuals {
                table.observe(r);
            }

            curr_row.clear();
            unfilter_row(&residuals, prev_row, filter, curr_row);

            prev_row.clear();
            prev_row.extend_from_slice(curr_row);
        }
    }
    Ok(())
}

/// Compress an 8-bit image row-by-row in streaming mode.
/// Memory consumption is O(width), strictly bounded regardless of image height.
pub fn compress_image_grayscale(
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Result<Vec<u8>, &'static str> {
    if pixels.len() != (width as usize * height as usize) {
        return Err("Pixel buffer length does not match width * height");
    }

    let mut out = Vec::with_capacity(pixels.len() / 2 + 64);
    out.extend_from_slice(b"TTHI");
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.push(1); // 1 channel

    let w = width as usize;
    let mut prev_row: Vec<u8> = Vec::new();
    let mut row_residuals = Vec::with_capacity(w);
    let mut table = AdaptiveTable::new_skewed();

    for y in 0..height as usize {
        let curr_row = &pixels[y * w..(y + 1) * w];
        encode_channel_row(curr_row, &mut prev_row, &mut table, &mut row_residuals, &mut out);
    }

    Ok(out)
}

/// Decompress an 8-bit grayscale image row-by-row.
pub fn decompress_image_grayscale(
    compressed: &[u8],
) -> Result<(u32, u32, Vec<u8>), &'static str> {
    if compressed.len() < 13 || &compressed[0..4] != b"TTHI" {
        return Err("Invalid Tether Image magic header");
    }

    let width = u32::from_le_bytes(compressed[4..8].try_into().unwrap());
    let height = u32::from_le_bytes(compressed[8..12].try_into().unwrap());
    let channels = compressed[12];
    if channels != 1 {
        return Err("Expected 1-channel grayscale image");
    }

    let w = width as usize;
    let h = height as usize;
    let mut pixels = Vec::with_capacity(w * h);
    let mut prev_row: Vec<u8> = Vec::new();
    let mut table = AdaptiveTable::new_skewed();
    let mut curr_row = Vec::with_capacity(w);

    let mut offset = 13;
    for _ in 0..h {
        decode_channel_row(w, compressed, &mut offset, &mut prev_row, &mut table, &mut curr_row)?;
        pixels.extend_from_slice(&curr_row);
    }

    Ok((width, height, pixels))
}

/// Compress an 8-bit RGB image row-by-row using reversible color decorrelation (G, R-G, B-G).
pub fn compress_image_rgb(
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Result<Vec<u8>, &'static str> {
    if pixels.len() != (width as usize * height as usize * 3) {
        return Err("Pixel buffer length does not match width * height * 3");
    }

    let mut out = Vec::with_capacity(pixels.len() / 2 + 64);
    out.extend_from_slice(b"TTHI");
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.push(3); // 3 channels

    let w = width as usize;
    let mut prev_g: Vec<u8> = Vec::new();
    let mut prev_r: Vec<u8> = Vec::new();
    let mut prev_b: Vec<u8> = Vec::new();

    let mut curr_g = Vec::with_capacity(w);
    let mut curr_r = Vec::with_capacity(w);
    let mut curr_b = Vec::with_capacity(w);

    let mut row_residuals = Vec::with_capacity(w);
    let mut table_g = AdaptiveTable::new_skewed();
    let mut table_r = AdaptiveTable::new_skewed();
    let mut table_b = AdaptiveTable::new_skewed();

    for y in 0..height as usize {
        let row_start = y * w * 3;
        let row_bytes = &pixels[row_start..row_start + w * 3];

        curr_g.clear();
        curr_r.clear();
        curr_b.clear();

        for i in 0..w {
            let r = row_bytes[i * 3];
            let g = row_bytes[i * 3 + 1];
            let b = row_bytes[i * 3 + 2];

            curr_g.push(g);
            curr_r.push(r.wrapping_sub(g));
            curr_b.push(b.wrapping_sub(g));
        }

        encode_channel_row(&curr_g, &mut prev_g, &mut table_g, &mut row_residuals, &mut out);
        encode_channel_row(&curr_r, &mut prev_r, &mut table_r, &mut row_residuals, &mut out);
        encode_channel_row(&curr_b, &mut prev_b, &mut table_b, &mut row_residuals, &mut out);
    }

    Ok(out)
}

/// Decompress an 8-bit RGB image row-by-row.
pub fn decompress_image_rgb(
    compressed: &[u8],
) -> Result<(u32, u32, Vec<u8>), &'static str> {
    if compressed.len() < 13 || &compressed[0..4] != b"TTHI" {
        return Err("Invalid Tether Image magic header");
    }

    let width = u32::from_le_bytes(compressed[4..8].try_into().unwrap());
    let height = u32::from_le_bytes(compressed[8..12].try_into().unwrap());
    let channels = compressed[12];
    if channels != 3 {
        return Err("Expected 3-channel RGB image");
    }

    let w = width as usize;
    let h = height as usize;
    let mut pixels = Vec::with_capacity(w * h * 3);

    let mut prev_g: Vec<u8> = Vec::new();
    let mut prev_r: Vec<u8> = Vec::new();
    let mut prev_b: Vec<u8> = Vec::new();

    let mut curr_g = Vec::with_capacity(w);
    let mut curr_r = Vec::with_capacity(w);
    let mut curr_b = Vec::with_capacity(w);

    let mut table_g = AdaptiveTable::new_skewed();
    let mut table_r = AdaptiveTable::new_skewed();
    let mut table_b = AdaptiveTable::new_skewed();

    let mut offset = 13;
    for _ in 0..h {
        decode_channel_row(w, compressed, &mut offset, &mut prev_g, &mut table_g, &mut curr_g)?;
        decode_channel_row(w, compressed, &mut offset, &mut prev_r, &mut table_r, &mut curr_r)?;
        decode_channel_row(w, compressed, &mut offset, &mut prev_b, &mut table_b, &mut curr_b)?;

        for i in 0..w {
            let g = curr_g[i];
            let r = curr_r[i].wrapping_add(g);
            let b = curr_b[i].wrapping_add(g);
            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
        }
    }

    Ok((width, height, pixels))
}

/// Unified image compressor supporting 1-channel (grayscale) and 3-channel (RGB).
pub fn compress_image(
    width: u32,
    height: u32,
    channels: u8,
    pixels: &[u8],
) -> Result<Vec<u8>, &'static str> {
    match channels {
        1 => compress_image_grayscale(width, height, pixels),
        3 => compress_image_rgb(width, height, pixels),
        _ => Err("Unsupported channel count (only 1 or 3 supported)"),
    }
}

/// Unified image decompressor supporting 1-channel (grayscale) and 3-channel (RGB).
pub fn decompress_image(
    compressed: &[u8],
) -> Result<(u32, u32, u8, Vec<u8>), &'static str> {
    if compressed.len() < 13 || &compressed[0..4] != b"TTHI" {
        return Err("Invalid Tether Image magic header");
    }
    let channels = compressed[12];
    match channels {
        1 => {
            let (w, h, p) = decompress_image_grayscale(compressed)?;
            Ok((w, h, 1, p))
        }
        3 => {
            let (w, h, p) = decompress_image_rgb(compressed)?;
            Ok((w, h, 3, p))
        }
        _ => Err("Unsupported channel count in header"),
    }
}
