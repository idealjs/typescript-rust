//! `--watch` mode: compile once, then watch the project directory for source
//! changes and recompile. Mirrors the Go oracle's watch loop using the portable
//! `notify::PollWatcher` (polling-based, so no platform-specific backend is
//! required).

use std::sync::mpsc::channel;
use std::time::Duration;

use notify::{Config, RecursiveMode, Watcher};

use super::{CommandLineResult, ExitStatus, System, perform_compilation};
use crate::core::compiler_options::CompilerOptions;
use crate::tsoptions::{ParsedCommandLine, get_parsed_command_line_of_config_file};

/// Run the compiler in watch mode.
///
/// Performs an initial compilation, sets up a file watcher on the directory
/// containing `tsconfig.json` (or the current directory when no config file is
/// present), and recompiles whenever a source file changes. The config file is
/// re-read on each recompilation so that edits to `tsconfig.json` take effect.
///
/// The loop runs until the process is interrupted (e.g. via Ctrl+C); there is
/// no graceful in-process shutdown, matching the Go implementation, which also
/// relies on process termination to exit watch mode.
pub(crate) fn watch_mode(
    sys: &dyn System,
    config: ParsedCommandLine,
    base_options: CompilerOptions,
    config_file_name: &str,
    pretty: bool,
) -> CommandLineResult {
    // 1. Initial compilation + start banner.
    {
        let mut writer = sys.writer();
        let _ = writeln!(writer);
        let _ = writeln!(
            writer,
            "[{}] Starting compilation in watch mode...",
            timestamp()
        );
    }
    let result = compile_once(sys, &config, &base_options, config_file_name, pretty);
    print_watch_summary(sys, result.status);

    // 2. Set up the file watcher on the project directory.
    let (tx, rx) = channel::<notify::Event>();
    let mut watcher = notify::PollWatcher::new(
        move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(1)),
    )
    .expect("failed to create file watcher");

    let watch_dir = std::path::Path::new(config_file_name)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    watcher
        .watch(watch_dir, RecursiveMode::Recursive)
        .expect("failed to watch project directory");

    // Keep the watcher alive for the lifetime of the event loop. (Note: a bare
    // `let _ = watcher;` would drop it immediately; a named binding does not.)
    let _watcher = watcher;

    // 3. Event loop: recompile on source-file changes.
    loop {
        match rx.recv() {
            Ok(event) => {
                let path_str = event.paths.first().and_then(|p| p.to_str()).unwrap_or("");
                if !is_source_file(path_str) {
                    continue;
                }

                // Debounce: wait briefly for a burst of events, then drain the
                // queue so a single save (which often triggers several events)
                // only causes one recompilation.
                std::thread::sleep(Duration::from_millis(100));
                while rx.try_recv().is_ok() {}

                {
                    let mut writer = sys.writer();
                    let _ = writeln!(writer);
                    let _ = writeln!(
                        writer,
                        "[{}] File change detected. Starting incremental compilation...",
                        timestamp()
                    );
                }
                let r = compile_once(sys, &config, &base_options, config_file_name, pretty);
                print_watch_summary(sys, r.status);
            }
            Err(_) => break,
        }
    }

    result
}

/// Perform a single compilation.
///
/// When a config file is present it is re-read (and merged with the saved
/// command-line `base_options`) so that edits to `tsconfig.json` are picked up.
/// When there is no config file, the already-parsed command-line config is
/// reused.
pub(super) fn compile_once(
    sys: &dyn System,
    config: &ParsedCommandLine,
    base_options: &CompilerOptions,
    config_file_name: &str,
    pretty: bool,
) -> CommandLineResult {
    let fresh = if !config_file_name.is_empty() {
        get_parsed_command_line_of_config_file(
            config_file_name,
            base_options,
            sys.current_directory(),
            sys.fs().as_ref(),
        )
    } else {
        config.clone()
    };
    perform_compilation(sys, fresh, pretty)
}

/// Print the post-compilation summary line shown after each compile in watch
/// mode.
pub(super) fn print_watch_summary(sys: &dyn System, status: ExitStatus) {
    let mut writer = sys.writer();
    if status == ExitStatus::Success {
        let _ = writeln!(
            writer,
            "[{}] Found 0 errors. Watching for file changes.",
            timestamp()
        );
    } else {
        let _ = writeln!(
            writer,
            "[{}] Found errors. Watching for file changes.",
            timestamp()
        );
    }
}

/// Whether a path refers to a TypeScript/JavaScript/JSON source file whose
/// change should trigger a recompilation.
pub(super) fn is_source_file(path: &str) -> bool {
    path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".js")
        || path.ends_with(".jsx")
        || path.ends_with(".json")
        || path.ends_with(".mts")
        || path.ends_with(".cts")
}

/// A compact `HH:MM:SS` timestamp (UTC) for watch-mode log lines.
pub(super) fn timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!(
        "{:02}:{:02}:{:02}",
        (now / 3600) % 24,
        (now / 60) % 60,
        now % 60
    )
}
