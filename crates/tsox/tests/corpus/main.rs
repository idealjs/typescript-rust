#[used]
#[unsafe(link_section = ".init_array")]
static ENSURE_MEMORY_LIMIT: extern "C" fn() = ensure_memory_limit;

extern "C" fn ensure_memory_limit() {
    const LIMIT_BYTES: u64 = 4 * 1024 * 1024 * 1024;
    let mut lim = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    let rc = unsafe { libc::getrlimit(libc::RLIMIT_AS, &mut lim) };
    let unlimited = rc == 0 && lim.rlim_cur == libc::RLIM_INFINITY;
    let over = rc == 0 && lim.rlim_cur > LIMIT_BYTES;
    if rc != 0 || unlimited || over {
        let cur = if rc == 0 {
            format!("{} bytes", lim.rlim_cur)
        } else {
            "getrlimit 失败".to_string()
        };
        eprintln!(
            "内存守卫:测试必须在 4GiB 内存限制下运行,当前 RLIMIT_AS = {cur}。\
             正确入口: (ulimit -v 4194304; cargo test --release --no-fail-fast ...)"
        );
        std::process::exit(101);
    }
}

mod submodule_transpile;
mod common;
mod submodule_compiler;
