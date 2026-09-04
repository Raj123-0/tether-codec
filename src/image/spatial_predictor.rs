// 2D Spatial causal predictors and row-by-row streaming image compression.

use crate::Vec;
use crate::entropy::{AdaptiveTable, RansEncoder, RansDecoder};
use crate::stream::bitstream::{BitReader, BitWriter};

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
    Raw = 9,
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
            9 => Some(Self::Raw),
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

/// Pack 8-bit residuals into symbols and extra bits.
/// Symbols:
///   0: single zero residual
///   1..127: small literal residuals 1..127 (0 extra bits)
///   128..191: large residuals 128..255 (prefix 128 + ((r - 128) >> 1), 1 extra bit)
///   192..255: runs of 2..65 zeros (192 -> 2 zeros, 255 -> 65 zeros, 0 extra bits)
pub fn pack_8bit_residuals(
    residuals: &[u8],
    symbols_out: &mut Vec<u8>,
    writer: &mut BitWriter,
) {
    let mut i = 0;
    let n = residuals.len();
    while i < n {
        let r = residuals[i];
        if r == 0 {
            let mut run = 0usize;
            while i < n && residuals[i] == 0 && run < 65 {
                run += 1;
                i += 1;
            }
            if run == 1 {
                symbols_out.push(0);
            } else {
                symbols_out.push(192 + (run - 2) as u8);
            }
        } else {
            if r < 128 {
                symbols_out.push(r);
            } else {
                let sym = 128 + ((r - 128) >> 1);
                symbols_out.push(sym);
                writer.write_bits(((r - 128) & 1) as u64, 1);
            }
            i += 1;
        }
    }
}

/// Unpack 8-bit residuals from symbols and extra bits.
pub fn unpack_8bit_residuals(
    symbols: &[u8],
    reader: &mut BitReader,
    target_count: usize,
    residuals_out: &mut Vec<u8>,
) -> Result<(), &'static str> {
    for &sym in symbols {
        if sym == 0 {
            residuals_out.push(0);
        } else if sym >= 192 {
            let run = (sym - 192) as usize + 2;
            if residuals_out.len() + run > target_count {
                return Err("Zero-run length exceeds target pixel count");
            }
            residuals_out.extend(core::iter::repeat(0).take(run));
        } else if sym < 128 {
            residuals_out.push(sym);
        } else {
            let base = 128 + ((sym - 128) << 1);
            let bit = reader.read_bits(1) as u8;
            residuals_out.push(base | bit);
        }
    }
    if residuals_out.len() != target_count {
        return Err("Decoded image residual count does not match expected row width");
    }
    Ok(())
}

/// Encode a single 8-bit image channel row with RepeatPrev, Constant, and Zero-Run fast paths.
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

    let mut symbols = Vec::with_capacity(curr_row.len());
    let mut bit_writer = BitWriter::with_capacity(curr_row.len() / 4 + 8);
    pack_8bit_residuals(row_residuals, &mut symbols, &mut bit_writer);
    let extra_bytes = bit_writer.finish();

    let mut rans = RansEncoder::new();
    rans.encode_block(&symbols, table);
    let payload = rans.finish();

    let compressed_len = 4 + extra_bytes.len() + payload.len();
    if compressed_len >= curr_row.len() {
        // Fall back to Raw row (FilterType::Raw)
        out.push(FilterType::Raw as u8);
        out.extend_from_slice(&(curr_row.len() as u32).to_le_bytes());
        out.extend_from_slice(curr_row);
    } else {
        for &s in symbols.iter() {
            table.observe(s);
        }
        out.push(filter as u8);
        out.extend_from_slice(&(compressed_len as u32).to_le_bytes());
        out.extend_from_slice(&(symbols.len() as u16).to_le_bytes());
        out.extend_from_slice(&(extra_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&extra_bytes);
        out.extend_from_slice(&payload);
    }

    prev_row.clear();
    prev_row.extend_from_slice(curr_row);
}

/// Decode a single 8-bit image channel row with run repeat support.
fn decode_channel_row_repeat(
    w: usize,
    compressed: &[u8],
    offset: &mut usize,
    prev_row: &mut Vec<u8>,
    table: &mut AdaptiveTable,
    curr_row: &mut Vec<u8>,
    remaining_repeats: &mut usize,
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
            if payload_len > 1 {
                *remaining_repeats = payload_len - 1;
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
        FilterType::Raw => {
            if *offset + payload_len > compressed.len() || payload_len != w {
                return Err("Truncated or invalid raw row payload");
            }
            curr_row.clear();
            curr_row.extend_from_slice(&compressed[*offset..*offset + payload_len]);
            *offset += payload_len;
            prev_row.clear();
            prev_row.extend_from_slice(curr_row);
        }
        _ => {
            if *offset + payload_len > compressed.len() {
                return Err("Truncated image row payload");
            }
            if payload_len < 4 {
                return Err("Invalid row payload length");
            }
            let sym_count = u16::from_le_bytes(compressed[*offset..*offset + 2].try_into().unwrap()) as usize;
            let extra_len = u16::from_le_bytes(compressed[*offset + 2..*offset + 4].try_into().unwrap()) as usize;
            *offset += 4;

            if 4 + extra_len > payload_len {
                return Err("Corrupted extra bits length in row payload");
            }
            let extra_bytes = &compressed[*offset..*offset + extra_len];
            *offset += extra_len;
            let rans_len = payload_len - 4 - extra_len;
            let rans_payload = &compressed[*offset..*offset + rans_len];
            *offset += rans_len;

            let mut rans = RansDecoder::new(rans_payload)?;
            let symbols = rans.decode_block(sym_count, table);

            for &s in &symbols {
                table.observe(s);
            }

            let mut bit_reader = BitReader::new(extra_bytes);
            let mut residuals = Vec::with_capacity(w);
            unpack_8bit_residuals(&symbols, &mut bit_reader, w, &mut residuals)?;

            curr_row.clear();
            unfilter_row(&residuals, prev_row, filter, curr_row);

            prev_row.clear();
            prev_row.extend_from_slice(curr_row);
        }
    }
    Ok(())
}

/// Decode a single 8-bit image channel row (calls repeat decoder with 0 remaining).
fn decode_channel_row(
    w: usize,
    compressed: &[u8],
    offset: &mut usize,
    prev_row: &mut Vec<u8>,
    table: &mut AdaptiveTable,
    curr_row: &mut Vec<u8>,
) -> Result<(), &'static str> {
    let mut dummy = 0;
    decode_channel_row_repeat(w, compressed, offset, prev_row, table, curr_row, &mut dummy)
}

/// Encode rows of an 8-bit image with RepeatPrev run-length acceleration and optional palette mapping.
fn encode_grayscale_stream(
    pixels: &[u8],
    w: usize,
    h: usize,
    lut: Option<&[u8; 256]>,
    out: &mut Vec<u8>,
) {
    let mut prev_row: Vec<u8> = Vec::new();
    let mut table = AdaptiveTable::new_skewed();
    let mut row_residuals = Vec::with_capacity(w);
    let mut mapped_row = Vec::with_capacity(w);

    let mut y = 0;
    while y < h {
        let curr_raw = &pixels[y * w..(y + 1) * w];
        let curr_slice: &[u8] = if let Some(lut) = lut {
            mapped_row.clear();
            for &b in curr_raw {
                mapped_row.push(lut[b as usize]);
            }
            &mapped_row
        } else {
            curr_raw
        };

        // Check RepeatPrev run
        if !prev_row.is_empty() && curr_slice == prev_row.as_slice() {
            let mut run = 0;
            while y < h {
                let next_raw = &pixels[y * w..(y + 1) * w];
                let matches = if let Some(lut) = lut {
                    next_raw.iter().enumerate().all(|(i, &b)| lut[b as usize] == prev_row[i])
                } else {
                    next_raw == prev_row.as_slice()
                };
                if matches {
                    run += 1;
                    y += 1;
                } else {
                    break;
                }
            }
            out.push(FilterType::RepeatPrev as u8);
            out.extend_from_slice(&(run as u32).to_le_bytes());
            continue;
        }

        encode_channel_row(curr_slice, &mut prev_row, &mut table, &mut row_residuals, out);
        y += 1;
    }
}

/// Compress an 8-bit image row-by-row in streaming mode with Auto-Palette support.
/// Memory consumption is O(width), strictly bounded regardless of image height.
pub fn compress_image_grayscale(
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Result<Vec<u8>, &'static str> {
    if pixels.len() != (width as usize * height as usize) {
        return Err("Pixel buffer length does not match width * height");
    }

    let w = width as usize;
    let h = height as usize;

    // Check for small palette (<= 16 unique colors, common in UI/charts)
    let mut seen = [false; 256];
    let mut palette = Vec::with_capacity(16);
    let mut too_many = false;
    for &p in pixels {
        if !seen[p as usize] {
            seen[p as usize] = true;
            palette.push(p);
            if palette.len() > 16 {
                too_many = true;
                break;
            }
        }
    }

    if !too_many && palette.len() >= 2 && palette.len() <= 16 {
        palette.sort_unstable();
        let mut lut = [0u8; 256];
        for (i, &c) in palette.iter().enumerate() {
            lut[c as usize] = i as u8;
        }

        let mut indexed_out = Vec::with_capacity(pixels.len() / 4 + 64);
        indexed_out.extend_from_slice(b"TTHI");
        indexed_out.extend_from_slice(&width.to_le_bytes());
        indexed_out.extend_from_slice(&height.to_le_bytes());
        indexed_out.push(0x81); // 1 channel with palette
        indexed_out.push(palette.len() as u8);
        indexed_out.extend_from_slice(&palette);

        encode_grayscale_stream(pixels, w, h, Some(&lut), &mut indexed_out);

        // Direct encoding comparison
        let mut direct_out = Vec::with_capacity(pixels.len() / 2 + 64);
        direct_out.extend_from_slice(b"TTHI");
        direct_out.extend_from_slice(&width.to_le_bytes());
        direct_out.extend_from_slice(&height.to_le_bytes());
        direct_out.push(1);

        encode_grayscale_stream(pixels, w, h, None, &mut direct_out);

        if indexed_out.len() <= direct_out.len() {
            return Ok(indexed_out);
        } else {
            return Ok(direct_out);
        }
    }

    let mut out = Vec::with_capacity(pixels.len() / 2 + 64);
    out.extend_from_slice(b"TTHI");
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.push(1); // 1 channel

    encode_grayscale_stream(pixels, w, h, None, &mut out);
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
    let channels_byte = compressed[12];
    if channels_byte != 1 && channels_byte != 0x81 {
        return Err("Expected 1-channel grayscale image");
    }

    let is_palette = channels_byte == 0x81;
    let mut offset = 13;
    let mut palette = Vec::new();
    if is_palette {
        if offset >= compressed.len() {
            return Err("Truncated palette header");
        }
        let pal_len = compressed[offset] as usize;
        offset += 1;
        if offset + pal_len > compressed.len() {
            return Err("Truncated palette entries");
        }
        palette.extend_from_slice(&compressed[offset..offset + pal_len]);
        offset += pal_len;
    }

    let w = width as usize;
    let h = height as usize;
    let mut pixels = Vec::with_capacity(w * h);
    let mut prev_row: Vec<u8> = Vec::new();
    let mut table = AdaptiveTable::new_skewed();
    let mut curr_row = Vec::with_capacity(w);
    let mut remaining_repeats = 0;

    for _ in 0..h {
        if remaining_repeats > 0 {
            remaining_repeats -= 1;
            curr_row.clear();
            curr_row.extend_from_slice(&prev_row);
        } else {
            decode_channel_row_repeat(w, compressed, &mut offset, &mut prev_row, &mut table, &mut curr_row, &mut remaining_repeats)?;
        }
        if is_palette {
            for b in curr_row.iter_mut() {
                let idx = *b as usize;
                if idx >= palette.len() {
                    return Err("Palette index out of bounds");
                }
                *b = palette[idx];
            }
        }
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
        1 | 0x81 => {
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
