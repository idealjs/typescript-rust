use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

struct CountingAlloc;

static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
static PEAK_LIVE: AtomicU64 = AtomicU64::new(0);

static BIG_ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
            let b = bucket_of(layout.size());
            BUCKET_COUNT[b].fetch_add(1, Ordering::Relaxed);
            BUCKET_BYTES[b].fetch_add(layout.size() as u64, Ordering::Relaxed);
            if layout.size() > 1_000_000 {
                BIG_ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            }
            let live = LIVE_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed) + layout.size() as u64;
            PEAK_LIVE.fetch_max(live, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE_BYTES.fetch_sub(layout.size() as u64, Ordering::Relaxed);
        System.dealloc(ptr, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            let diff = new_size as i64 - layout.size() as i64;
            if diff > 0 {
                ALLOC_BYTES.fetch_add(diff as u64, Ordering::Relaxed);
            }
            let live = if diff >= 0 {
                LIVE_BYTES.fetch_add(diff as u64, Ordering::Relaxed) as i64 + diff
            } else {
                LIVE_BYTES.fetch_add(diff.unsigned_abs(), Ordering::Relaxed) as i64 + diff
            };
            PEAK_LIVE.fetch_max(live.max(0) as u64, Ordering::Relaxed);
        }
        new_ptr
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

static BUCKET_COUNT: [AtomicU64; 6] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];
static BUCKET_BYTES: [AtomicU64; 6] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

fn bucket_of(size: usize) -> usize {
    match size {
        0..=63 => 0,
        64..=255 => 1,
        256..=4095 => 2,
        4096..=65535 => 3,
        65536..=1048575 => 4,
        _ => 5,
    }
}

#[test]
fn probe_lib_dom_parse_allocations() {
    let path = "bundled/libs/lib.dom.d.ts";
    let text = std::fs::read_to_string(path).expect("lib.dom.d.ts not found");
    println!("text bytes: {}", text.len());

    let c0 = ALLOC_COUNT.load(Ordering::Relaxed);
    let b0 = ALLOC_BYTES.load(Ordering::Relaxed);
    let t0 = std::time::Instant::now();
    let sf = tsox::parser::Parser::parse_source_file_text("lib.dom.d.ts", text);
    let elapsed = t0.elapsed();

    let allocs = ALLOC_COUNT.load(Ordering::Relaxed) - c0;
    let bytes = ALLOC_BYTES.load(Ordering::Relaxed) - b0;
    let live = LIVE_BYTES.load(Ordering::Relaxed);
    let peak = PEAK_LIVE.load(Ordering::Relaxed);

    println!(
        "parse time: {:?}\nallocations: {}\nallocated bytes: {} ({:.1} MB)\nlive bytes (retained AST + leftovers): {} ({:.1} MB)\npeak live: {:.1} MB",
        elapsed,
        allocs,
        bytes,
        bytes as f64 / 1e6,
        live,
        live as f64 / 1e6,
        peak as f64 / 1e6,
    );
    let limits = ["<=63B", "64-255B", "256B-4K", "4K-64K", "64K-1M", ">1M"];
    for i in 0..6 {
        println!(
            "bucket {:>8}: count={:>9} bytes={:>12}",
            limits[i],
            BUCKET_COUNT[i].load(Ordering::Relaxed) - if i == 0 { 0 } else { 0 },
            BUCKET_BYTES[i].load(Ordering::Relaxed),
        );
    }
    let _ = sf;
}
