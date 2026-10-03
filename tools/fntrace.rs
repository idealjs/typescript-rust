//! 全函数调用序列记录（执行栈采集）。
//! 插桩脚本在每个函数体首行插入 `fntrace::enter("<函数名>")`；
//! env TSOX_FN_TRACE=1 时记录调用序列，由调用方在用例结束时显式
//! flush 到目标文件。未开启时每次调用仅一次原子读。
//! enter 内联去重保序：仅首次调用的函数入序列，内存只与唯一函数数
//! 相关（千级），与总调用次数无关——多 worker 并发下无内存膨胀。

use std::collections::HashSet;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static ENABLED: AtomicBool = AtomicBool::new(false);

struct TraceState {
    seen: HashSet<&'static str>,
    order: Vec<&'static str>,
}

fn state() -> &'static Mutex<TraceState> {
    static SEQ: std::sync::OnceLock<Mutex<TraceState>> = std::sync::OnceLock::new();
    SEQ.get_or_init(|| {
        Mutex::new(TraceState {
            seen: HashSet::new(),
            order: Vec::new(),
        })
    })
}

pub fn maybe_init() {
    if let Ok(v) = std::env::var("TSOX_FN_TRACE") {
        if !v.is_empty() {
            ENABLED.store(true, Ordering::Relaxed);
        }
    }
}

#[inline(always)]
pub fn enter(name: &'static str) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    if let Ok(mut s) = state().lock() {
        if s.seen.insert(name) {
            s.order.push(name);
        }
    }
}

/// 首次调用顺序的唯一函数序列，一行一个函数名。
pub fn flush_to(path: &str) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let snapshot = match state().lock() {
        Ok(s) => s.order.clone(),
        Err(_) => return,
    };
    if let Ok(f) = std::fs::File::create(path) {
        let mut out = std::io::BufWriter::new(f);
        for name in snapshot {
            if writeln!(out, "{name}").is_err() {
                return;
            }
        }
    }
}
