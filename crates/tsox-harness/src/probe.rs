use std::cell::Cell;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub enum Slot {
    Read,
    Mount,
    FsBuild,
    Program,
    Check,
    Types,
    Render,
}

thread_local! {
    static READ: Cell<u64> = const { Cell::new(0) };
    static MOUNT: Cell<u64> = const { Cell::new(0) };
    static FS_BUILD: Cell<u64> = const { Cell::new(0) };
    static PROGRAM: Cell<u64> = const { Cell::new(0) };
    static CHECK: Cell<u64> = const { Cell::new(0) };
    static TYPES: Cell<u64> = const { Cell::new(0) };
    static RENDER: Cell<u64> = const { Cell::new(0) };
}

pub struct Phase {
    slot: Slot,
    start: Instant,
}

pub fn enter(slot: Slot) -> Option<Phase> {
    if enabled() {
        Some(Phase {
            slot,
            start: Instant::now(),
        })
    } else {
        None
    }
}

impl Drop for Phase {
    fn drop(&mut self) {
        add(self.slot, self.start.elapsed());
    }
}

pub fn add(slot: Slot, elapsed: Duration) {
    if !enabled() {
        return;
    }
    let nanos = elapsed.as_nanos() as u64;
    let cell = match slot {
        Slot::Read => &READ,
        Slot::Mount => &MOUNT,
        Slot::FsBuild => &FS_BUILD,
        Slot::Program => &PROGRAM,
        Slot::Check => &CHECK,
        Slot::Types => &TYPES,
        Slot::Render => &RENDER,
    };
    cell.with(|c| c.set(c.get() + nanos));
}

pub fn emit_case(basename: &str) {
    if !enabled() {
        return;
    }
    let read = READ.replace(0);
    let mount = MOUNT.replace(0);
    let fsbuild = FS_BUILD.replace(0);
    let program = PROGRAM.replace(0);
    let check = CHECK.replace(0);
    let types = TYPES.replace(0);
    let render = RENDER.replace(0);
    let total = read + mount + fsbuild + program + check + types + render;
    let ms = |nanos: u64| nanos as f64 / 1_000_000.0;
    let line = format!(
        "case={basename} read_ms={:.3} mount_ms={:.3} fsbuild_ms={:.3} \
         program_ms={:.3} check_ms={:.3} types_ms={:.3} render_ms={:.3} total_ms={:.3}",
        ms(read),
        ms(mount),
        ms(fsbuild),
        ms(program),
        ms(check),
        ms(types),
        ms(render),
        ms(total),
    );
    if let Some(path) = std::env::var_os("TSOX_PROBE_PHASES") {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            use std::io::Write as _;
            let _ = writeln!(f, "{line}");
        }
    }
}

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("TSOX_PROBE_PHASES").is_some())
}
