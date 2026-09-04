// Explicit, enforced memory contract for Tether encoder and decoder.

/// Default hard memory ceiling: 32 KB
pub const MAX_ENCODER_MEMORY_BYTES: usize = 32_768;
pub const MAX_DECODER_MEMORY_BYTES: usize = 32_768;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MemoryProfile {
    /// Ultra-constrained MCU profile: max 4 KB working RAM
    Micro4KB,
    /// Constrained embedded profile: max 16 KB working RAM
    Embedded16KB,
    /// Standard embedded profile (default): max 32 KB working RAM
    Embedded32KB,
    /// Desktop-class profile: max 64 KB working RAM
    Desktop64KB,
}

#[derive(Copy, Clone, Debug)]
pub struct MemoryBudget {
    pub profile: MemoryProfile,
    pub max_encoder_bytes: usize,
    pub max_decoder_bytes: usize,
    pub block_samples: usize,
}

impl MemoryBudget {
    pub fn micro_4kb() -> Self {
        Self {
            profile: MemoryProfile::Micro4KB,
            max_encoder_bytes: 4_096,
            max_decoder_bytes: 4_096,
            block_samples: 128,
        }
    }

    pub fn embedded_16kb() -> Self {
        Self {
            profile: MemoryProfile::Embedded16KB,
            max_encoder_bytes: 16_384,
            max_decoder_bytes: 16_384,
            block_samples: 256,
        }
    }

    pub fn embedded_32kb() -> Self {
        Self {
            profile: MemoryProfile::Embedded32KB,
            max_encoder_bytes: 32_768,
            max_decoder_bytes: 32_768,
            block_samples: 256,
        }
    }

    pub fn desktop_64kb() -> Self {
        Self {
            profile: MemoryProfile::Desktop64KB,
            max_encoder_bytes: 65_536,
            max_decoder_bytes: 65_536,
            block_samples: 512,
        }
    }

    /// Assert that a requested working memory size strictly respects the budget.
    #[inline]
    pub fn check_encoder_bound(&self, bytes_allocated: usize) -> Result<(), &'static str> {
        if bytes_allocated > self.max_encoder_bytes {
            Err("Encoder working set exceeded configured MemoryBudget ceiling")
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn check_decoder_bound(&self, bytes_allocated: usize) -> Result<(), &'static str> {
        if bytes_allocated > self.max_decoder_bytes {
            Err("Decoder working set exceeded configured MemoryBudget ceiling")
        } else {
            Ok(())
        }
    }
}

impl Default for MemoryBudget {
    fn default() -> Self {
        Self::embedded_32kb()
    }
}
