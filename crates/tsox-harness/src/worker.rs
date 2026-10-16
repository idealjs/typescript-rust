use std::io::BufRead;
use std::path::Path;

use tsox_core::fntrace;

use crate::case_run::{payload_entries, process_case};

pub fn run_worker(case_path: &Path, out_path: &Path) {
    fntrace::maybe_init();
    let payload = process_payload(case_path);
    flush_trace();
    let _ = std::fs::write(out_path, payload);
}

/// 常驻模式:stdin 每行一个请求 `<case>\t<out>\t<fn_trace>\t<types_stem>\t<types_header>`,
/// 处理完向 stdout 回 `DONE`,EOF 退出。每例开始前重置跨例状态(池化语义的根基)。
pub fn run_resident() {
    fntrace::maybe_init();
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        let [case, out, fn_trace, types_stem, types_header] = parts.as_slice() else {
            eprintln!("resident: malformed request: {line:?}");
            continue;
        };
        // 上一例的大栈线程已 join,此刻进程内无并发线程,env 写入是安全的
        unsafe {
            std::env::set_var("TSOX_FN_TRACE", fn_trace);
            std::env::set_var("TSOX_TYPES_STEM", types_stem);
            std::env::set_var("TSOX_TYPES_HEADER", types_header);
        }
        reset_per_case_state();
        // 归还上一例释放的页给 OS:滞留高水位会叠加到下一重例,打穿虚拟上限
        unsafe { libc::malloc_trim(0) };
        report_resident_rss(case);
        let payload = process_payload(Path::new(case));
        flush_trace();
        let _ = std::fs::write(out, payload);
        println!("DONE");
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }
}

fn report_resident_rss(case: &str) {
    let mut rss = String::new();
    let mut size = String::new();
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                rss = line["VmRSS:".len()..].trim().to_string();
            } else if line.starts_with("VmSize:") {
                size = line["VmSize:".len()..].trim().to_string();
            }
        }
    }
    let basename = case.rsplit('/').next().unwrap_or(case);
    let a = crate::alloc::ALLOC_COUNT.load(std::sync::atomic::Ordering::Relaxed);
    let b = crate::alloc::ALLOC_BYTES.load(std::sync::atomic::Ordering::Relaxed);
    eprintln!("RESIDENT case={basename} rss={rss} vsize={size} allocs={a} bytes={b}");
}

fn reset_per_case_state() {
    tsox_frontend::reset_per_case_state();
    tsox_checker::reset_per_case_state();
    tsox_compile::reset_per_case_state();
    tsox_tsoptions::reset_per_case_state();
}

fn process_payload(case_path: &Path) -> String {
    let case_path = case_path.to_path_buf();
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || {
            let basename = case_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            let read = {
                let read_phase = crate::probe::enter(crate::probe::Slot::Read);
                std::fs::read_to_string(&case_path)
            };
            match read {
                Ok(content) => {
                    let configs = process_case(&content, &basename);
                    serde_json::to_string(&payload_entries(&configs))
                        .unwrap_or_else(|_| "[]".to_string())
                }
                Err(e) => {
                    crate::probe::emit_case(&basename);
                    serde_json::json!([{
                        "suffix": "",
                        "skip": format!("unreadable as UTF-8: {e}")
                    }])
                    .to_string()
                }
            }
        })
        .expect("spawn worker compile thread")
        .join()
        .unwrap_or_else(|_| "[]".to_string())
}

fn flush_trace() {
    if let Ok(p) = std::env::var("TSOX_FN_TRACE") {
        if !p.is_empty() {
            fntrace::flush_to(&p);
        }
    }
}
