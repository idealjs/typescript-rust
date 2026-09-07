use std::sync::mpsc::channel;
use std::time::Duration;

use notify::{Config, RecursiveMode, Watcher};

use super::{CommandLineResult, ExitStatus, System, perform_compilation};
use crate::core::compiler_options::CompilerOptions;
use crate::locale::Locale;
use crate::tsoptions::{ParsedCommandLine, get_parsed_command_line_of_config_file};

pub(crate) fn watch_mode(
    sys: &dyn System,
    config: ParsedCommandLine,
    base_options: CompilerOptions,
    config_file_name: &str,
    pretty: bool,
    locale: Option<Locale>,
) -> CommandLineResult {
    {
        let mut writer = sys.writer();
        let _ = writeln!(writer);
        let _ = writeln!(
            writer,
            "[{}] Starting compilation in watch mode...",
            timestamp()
        );
    }
    let result = compile_once(
        sys,
        &config,
        &base_options,
        config_file_name,
        pretty,
        locale.clone(),
    );
    print_watch_summary(sys, result.status);

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

    let _watcher = watcher;

    loop {
        match rx.recv() {
            Ok(event) => {
                let path_str = event.paths.first().and_then(|p| p.to_str()).unwrap_or("");
                if !is_source_file(path_str) {
                    continue;
                }

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
                let r = compile_once(
                    sys,
                    &config,
                    &base_options,
                    config_file_name,
                    pretty,
                    locale.clone(),
                );
                print_watch_summary(sys, r.status);
            }
            Err(_) => break,
        }
    }

    result
}

pub(super) fn compile_once(
    sys: &dyn System,
    config: &ParsedCommandLine,
    base_options: &CompilerOptions,
    config_file_name: &str,
    pretty: bool,
    locale: Option<Locale>,
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
    perform_compilation(sys, fresh, pretty, locale.as_ref())
}

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

pub(super) fn is_source_file(path: &str) -> bool {
    path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".js")
        || path.ends_with(".jsx")
        || path.ends_with(".json")
        || path.ends_with(".mts")
        || path.ends_with(".cts")
}

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
