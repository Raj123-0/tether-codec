// Tether: Low-memory, high-speed lossless compression for streaming numeric data.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
pub(crate) use alloc::vec::Vec;

#[cfg(feature = "std")]
pub(crate) use std::vec::Vec;

pub mod entropy;
pub mod predictor;
pub mod stream;
pub mod memory_budget;
pub mod image;

pub use memory_budget::{MemoryBudget, MemoryProfile};
pub use stream::block_writer::BlockWriter;
pub use stream::block_reader::BlockReader;
pub use stream::{MAGIC, StreamDataType};
pub use predictor::PredictorMode;
pub use entropy::AdaptiveTable;

/// Compress a raw byte stream into Tether format under the given memory budget.
pub fn compress(data: &[u8], budget: MemoryBudget) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() / 2 + 32);
    out.extend_from_slice(MAGIC);
    let profile_byte = match budget.profile {
        MemoryProfile::Micro4KB => 0,
        MemoryProfile::Embedded16KB => 1,
        MemoryProfile::Embedded32KB => 2,
        MemoryProfile::Desktop64KB => 3,
    };
    out.push(profile_byte);

    let num_words = data.len() / 8;
    let rem_bytes = data.len() % 8;

    out.push(StreamDataType::I64 as u8);
    out.push(rem_bytes as u8);

    let mut table = AdaptiveTable::new_skewed();
    let mut words = Vec::with_capacity(num_words);
    for i in 0..num_words {
        let start = i * 8;
        let w = i64::from_le_bytes(data[start..start + 8].try_into().unwrap());
        words.push(w);
    }

    if !words.is_empty() {
        let mut initial_sample = words[0];
        let mut initial_history = [words[0]; 3];

        for chunk in words.chunks(budget.block_samples) {
            let (s, h) = BlockWriter::write_auto_i64_block(chunk, initial_sample, initial_history, &mut table, &mut out);
            initial_sample = s;
            initial_history = h;
        }
    }

    if rem_bytes > 0 {
        let tail_start = num_words * 8;
        out.extend_from_slice(&data[tail_start..]);
    }

    out
}

/// Decompress a Tether compressed byte stream.
pub fn decompress(compressed: &[u8]) -> Result<Vec<u8>, &'static str> {
    if compressed.len() < 7 {
        return Err("Payload too short for Tether header");
    }
    if &compressed[0..4] != MAGIC {
        return Err("Invalid Tether magic bytes");
    }
    let _profile_byte = compressed[4];
    let data_type_byte = compressed[5];
    let rem_bytes = compressed[6] as usize;

    let _data_type = StreamDataType::from_u8(data_type_byte).ok_or("Unknown stream data type")?;

    let mut table = AdaptiveTable::new_skewed();
    let mut offset = 7;
    let mut decoded_u64 = Vec::new();

    while offset < compressed.len().saturating_sub(rem_bytes) {
        let progressed = BlockReader::read_block_u64(compressed, &mut offset, &mut table, &mut decoded_u64)?;
        if !progressed {
            break;
        }
    }

    let mut out = Vec::with_capacity(decoded_u64.len() * 8 + rem_bytes);
    for val in decoded_u64 {
        out.extend_from_slice(&val.to_le_bytes());
    }

    if rem_bytes > 0 && offset < compressed.len() {
        let tail = &compressed[offset..offset + rem_bytes.min(compressed.len() - offset)];
        out.extend_from_slice(tail);
    }

    Ok(out)
}

/// Compress an i64 numeric time-series slice.
pub fn compress_i64(samples: &[i64], budget: MemoryBudget) -> Vec<u8> {
    let bytes = unsafe { core::slice::from_raw_parts(samples.as_ptr() as *const u8, samples.len() * 8) };
    compress(bytes, budget)
}

/// Decompress into an i64 numeric time-series slice.
pub fn decompress_i64(compressed: &[u8]) -> Result<Vec<i64>, &'static str> {
    let bytes = decompress(compressed)?;
    let num_samples = bytes.len() / 8;
    let mut out = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let start = i * 8;
        out.push(i64::from_le_bytes(bytes[start..start + 8].try_into().unwrap()));
    }
    Ok(out)
}

/// Compress an f64 floating-point time-series slice using Gorilla XOR mode.
pub fn compress_f64(samples: &[f64], budget: MemoryBudget) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 4 + 32);
    out.extend_from_slice(MAGIC);
    let profile_byte = match budget.profile {
        MemoryProfile::Micro4KB => 0,
        MemoryProfile::Embedded16KB => 1,
        MemoryProfile::Embedded32KB => 2,
        MemoryProfile::Desktop64KB => 3,
    };
    out.push(profile_byte);
    out.push(StreamDataType::F64 as u8);
    out.push(0);

    let u64_samples: Vec<u64> = samples.iter().map(|&f| f.to_bits()).collect();
    if !u64_samples.is_empty() {
        let mut table = AdaptiveTable::new_skewed();
        let mut initial = u64_samples[0];
        for chunk in u64_samples.chunks(budget.block_samples) {
            BlockWriter::write_xor_u64_block(chunk, initial, &mut table, &mut out);
            initial = *chunk.last().unwrap();
        }
    }
    out
}

/// Decompress into an f64 floating-point time-series slice.
pub fn decompress_f64(compressed: &[u8]) -> Result<Vec<f64>, &'static str> {
    if compressed.len() < 7 {
        return Err("Payload too short for Tether header");
    }
    if &compressed[0..4] != MAGIC {
        return Err("Invalid Tether magic bytes");
    }
    let mut table = AdaptiveTable::new_skewed();
    let mut offset = 7;
    let mut decoded_u64 = Vec::new();
    while offset < compressed.len() {
        let progressed = BlockReader::read_block_u64(compressed, &mut offset, &mut table, &mut decoded_u64)?;
        if !progressed {
            break;
        }
    }
    Ok(decoded_u64.into_iter().map(f64::from_bits).collect())
}

#[cfg(feature = "std")]
pub mod streaming {
    use super::*;
    use std::io::{Write, Result as IoResult};

    /// Incremental chunk-by-chunk streaming compressor.
    pub struct TetherWriter<W: Write> {
        writer: W,
        budget: MemoryBudget,
        buffer: Vec<i64>,
        table: AdaptiveTable,
        initial_sample: i64,
        initial_history: [i64; 3],
        header_written: bool,
    }

    impl<W: Write> TetherWriter<W> {
        pub fn new(writer: W, budget: MemoryBudget) -> Self {
            Self {
                writer,
                budget,
                buffer: Vec::with_capacity(budget.block_samples),
                table: AdaptiveTable::new_skewed(),
                initial_sample: 0,
                initial_history: [0; 3],
                header_written: false,
            }
        }

        fn ensure_header(&mut self) -> IoResult<()> {
            if !self.header_written {
                self.writer.write_all(MAGIC)?;
                let profile_byte = match self.budget.profile {
                    MemoryProfile::Micro4KB => 0,
                    MemoryProfile::Embedded16KB => 1,
                    MemoryProfile::Embedded32KB => 2,
                    MemoryProfile::Desktop64KB => 3,
                };
                self.writer.write_all(&[profile_byte, StreamDataType::I64 as u8, 0])?;
                self.header_written = true;
            }
            Ok(())
        }

        pub fn write_sample(&mut self, sample: i64) -> IoResult<()> {
            self.ensure_header()?;
            if self.buffer.is_empty() && self.initial_sample == 0 {
                self.initial_sample = sample;
                self.initial_history = [sample; 3];
            }
            self.buffer.push(sample);
            if self.buffer.len() >= self.budget.block_samples {
                self.flush_block()?;
            }
            Ok(())
        }

        pub fn write_samples(&mut self, samples: &[i64]) -> IoResult<()> {
            for &s in samples {
                self.write_sample(s)?;
            }
            Ok(())
        }

        pub fn flush_block(&mut self) -> IoResult<()> {
            if self.buffer.is_empty() {
                return Ok(());
            }
            let mut block_bytes = Vec::new();
            let (s, h) = BlockWriter::write_auto_i64_block(
                &self.buffer,
                self.initial_sample,
                self.initial_history,
                &mut self.table,
                &mut block_bytes,
            );
            self.initial_sample = s;
            self.initial_history = h;
            self.buffer.clear();
            self.writer.write_all(&block_bytes)?;
            Ok(())
        }

        pub fn finish(mut self) -> IoResult<W> {
            self.ensure_header()?;
            self.flush_block()?;
            self.writer.flush()?;
            Ok(self.writer)
        }
    }
}
