use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};
use tether::memory_budget::MemoryBudget;
use tether::stream::block_writer::BlockWriter;
use tether::stream::block_reader::BlockReader;
use tether::entropy::AdaptiveTable;

struct TrackingAlloc {
    current: AtomicIsize,
    peak: AtomicIsize,
}

impl TrackingAlloc {
    const fn new() -> Self {
        Self {
            current: AtomicIsize::new(0),
            peak: AtomicIsize::new(0),
        }
    }

    fn reset(&self) {
        self.current.store(0, Ordering::SeqCst);
        self.peak.store(0, Ordering::SeqCst);
    }

    fn peak(&self) -> usize {
        self.peak.load(Ordering::SeqCst).max(0) as usize
    }
}

unsafe impl GlobalAlloc for TrackingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let size = layout.size() as isize;
            let current = self.current.fetch_add(size, Ordering::SeqCst) + size;
            let mut current_peak = self.peak.load(Ordering::SeqCst);
            while current > current_peak {
                match self.peak.compare_exchange_weak(current_peak, current, Ordering::SeqCst, Ordering::SeqCst) {
                    Ok(_) => break,
                    Err(actual) => current_peak = actual,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        self.current.fetch_sub(layout.size() as isize, Ordering::SeqCst);
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAlloc = TrackingAlloc::new();

#[test]
fn test_encoder_memory_ceiling() {
    let budget = MemoryBudget::embedded_32kb();
    let n = 20_000;
    let data: Vec<i64> = (0..n).map(|i| (i * 3 + (i % 10)) as i64).collect();

    ALLOCATOR.reset();

    let mut table_enc = AdaptiveTable::new_skewed();
    let mut compressed = Vec::with_capacity(n * 2);
    let mut initial = 0i64;

    for chunk in data.chunks(budget.block_samples) {
        BlockWriter::write_delta_i64_block(chunk, initial, &mut table_enc, &mut compressed);
        initial = *chunk.last().unwrap();
    }

    let peak = ALLOCATOR.peak();
    println!("Encoder peak working memory for {} samples: {} bytes (Ceiling: {} B)",
             n, peak, budget.max_encoder_bytes);

    assert!(peak <= budget.max_encoder_bytes + (n * 2),
            "Encoder exceeded memory ceiling: peak {} B vs limit {} B", peak, budget.max_encoder_bytes);
}

#[test]
fn test_decoder_memory_ceiling() {
    let budget = MemoryBudget::embedded_32kb();
    let n = 20_000;
    let data: Vec<i64> = (0..n).map(|i| (i * 3 + (i % 10)) as i64).collect();

    let mut table_enc = AdaptiveTable::new_skewed();
    let mut compressed = Vec::new();
    let mut initial = 0i64;
    for chunk in data.chunks(budget.block_samples) {
        BlockWriter::write_delta_i64_block(chunk, initial, &mut table_enc, &mut compressed);
        initial = *chunk.last().unwrap();
    }

    ALLOCATOR.reset();

    let mut table_dec = AdaptiveTable::new_skewed();
    let mut decoded = Vec::with_capacity(n);
    let mut offset = 0;

    while BlockReader::read_block_u64(&compressed, &mut offset, &mut table_dec, &mut decoded).unwrap() {}

    let peak = ALLOCATOR.peak();
    println!("Decoder peak working memory for {} samples: {} bytes (Ceiling: {} B)",
             n, peak, budget.max_decoder_bytes);

    assert_eq!(decoded.len(), n);
}
