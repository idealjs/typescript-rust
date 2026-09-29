#![allow(dead_code, unused_imports, unused_variables)]

use std::io::{Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

pub struct ApiFlags {
    pub cwd: String,
    pub pipe_path: String,
    pub callbacks: String,
    pub is_async: bool,
    pub timing: bool,
    pub run_external_code: bool,
}

impl Default for ApiFlags {
    fn default() -> Self {
        Self {
            cwd: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            pipe_path: String::new(),
            callbacks: String::new(),
            is_async: false,
            timing: false,
            run_external_code: false,
        }
    }
}

pub fn parse_api_flags(args: &[String]) -> Result<ApiFlags, String> {
    let mut result = ApiFlags::default();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        let (name, inline_value) = match arg.strip_prefix("--") {
            Some(rest) => match rest.split_once('=') {
                Some((n, v)) => (n.to_string(), Some(v.to_string())),
                None => (rest.to_string(), None),
            },
            None => {
                i += 1;
                continue;
            }
        };
        let mut next = |i: &mut usize| -> Option<String> {
            if let Some(v) = &inline_value {
                return Some(v.clone());
            }
            *i += 1;
            args.get(*i).cloned()
        };
        match name.as_str() {
            "cwd" => result.cwd = next(&mut i).unwrap_or(result.cwd),
            "pipe" => result.pipe_path = next(&mut i).unwrap_or_default(),
            "callbacks" => result.callbacks = next(&mut i).unwrap_or_default(),
            "async" => result.is_async = true,
            "timing" => result.timing = true,
            "runExternalCode" => result.run_external_code = true,
            _ => {}
        }
        i += 1;
    }
    Ok(result)
}

pub fn run_api(args: &[String]) -> i32 {
    let flags = match parse_api_flags(args) {
        Ok(flags) => flags,
        Err(_) => return 2,
    };
    let default_library_path = tsox_checker::bundled::lib_path();
    let callbacks_list: Vec<String> = if flags.callbacks.is_empty() {
        Vec::new()
    } else {
        flags.callbacks.split(',').map(|s| s.to_string()).collect()
    };
    let _options = (flags.cwd.clone(), default_library_path, callbacks_list, flags.is_async, flags.timing, flags.run_external_code, flags.pipe_path.clone());
    0
}

pub fn run_lsp(args: &[String]) -> i32 {
    let mut stdio = false;
    let mut pprof_dir = String::new();
    let mut client_process_id: i64 = 0;
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if let Some(rest) = arg.strip_prefix("--") {
            let (name, value) = match rest.split_once('=') {
                Some((n, v)) => (n, Some(v.to_string())),
                None => (rest, None),
            };
            match name {
                "stdio" => stdio = true,
                "pprofDir" => pprof_dir = value.unwrap_or_default(),
                "clientProcessId" => {
                    client_process_id = value
                        .unwrap_or_else(|| {
                            i += 1;
                            args.get(i).cloned().unwrap_or_default()
                        })
                        .parse()
                        .unwrap_or(0);
                }
                _ => {}
            }
        }
        i += 1;
    }
    if !stdio {
        eprintln!("only stdio is supported");
        return 1;
    }
    0
}

pub fn new_parent_process_watchdog(
    client_process_id: i64,
) -> Option<Arc<dyn Fn(i64) + Send + Sync>> {
    if !process_alive_supported() {
        return None;
    }
    if client_process_id > 0 {
        start_parent_process_watchdog(client_process_id);
        return None;
    }
    Some(Arc::new(|parent_pid| start_parent_process_watchdog(parent_pid)))
}

pub fn start_parent_process_watchdog(parent_pid: i64) {
    if parent_pid <= 0 {
        return;
    }
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(5));
            if !is_process_alive(parent_pid as i32) {
                eprintln!("Parent process {} has exited, shutting down.", parent_pid);
                return;
            }
        }
    });
}

pub fn run_main(args: &[String]) -> i32 {
    if let Some(first) = args.first() {
        match first.as_str() {
            "--lsp" => return run_lsp(&args[1..]),
            "--api" => return run_api(&args[1..]),
            _ => {}
        }
    }
    let sys = new_system();
    let result = crate::execute::command_line(&sys, args);
    result.status as i32
}

#[cfg(unix)]
pub fn process_alive_supported() -> bool {
    true
}

#[cfg(unix)]
pub fn is_process_alive(pid: i32) -> bool {
    use std::os::unix::process::ExitStatusExt;
    let Ok(status) = std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    else {
        return false;
    };
    status.code() == Some(0) || status.signal() == Some(0)
}

#[cfg(windows)]
pub fn process_alive_supported() -> bool {
    true
}

#[cfg(windows)]
pub fn is_process_alive(pid: i32) -> bool {
    false
}

#[cfg(not(any(unix, windows)))]
pub fn process_alive_supported() -> bool {
    false
}

#[cfg(not(any(unix, windows)))]
pub fn is_process_alive(_pid: i32) -> bool {
    panic!("isProcessAlive is not supported on this platform")
}

#[cfg(windows)]
pub fn enable_virtual_terminal_processing_init() {
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
        STD_OUTPUT_HANDLE,
    };
    unsafe {
        let h = GetStdHandle(STD_OUTPUT_HANDLE);
        if h.is_null() {
            return;
        }
        let mut mode: u32 = 0;
        if GetConsoleMode(h, &mut mode) == 0 {
            return;
        }
        if mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING == 0 {
            let _ = SetConsoleMode(h, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
    }
}

#[cfg(not(windows))]
pub fn enable_virtual_terminal_processing_init() {}

pub struct ChildProcess {
    pub child: Child,
    pub stdin: Option<ChildStdin>,
    pub stdout: Option<ChildStdout>,
}

impl ChildProcess {
    pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.stdout {
            Some(stdout) => stdout.read(buf),
            None => Ok(0),
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.stdin {
            Some(stdin) => stdin.write(buf),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "child stdin closed",
            )),
        }
    }

    pub fn exit_code(&mut self) -> Option<i32> {
        self.child.try_wait().ok().flatten().and_then(|s| s.code())
    }

    pub fn close(&mut self) -> std::io::Result<()> {
        if let Some(stdin) = self.stdin.take() {
            drop(stdin);
        }
        let _ = self.child.kill();
        self.child.wait()?;
        Ok(())
    }
}

pub fn spawn_process(
    command: &[String],
    dir: &str,
    stderr: Box<dyn Write + Send>,
) -> std::io::Result<ChildProcess> {
    let Some((program, rest)) = command.split_first() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty command",
        ));
    };
    let mut cmd = Command::new(program);
    cmd.args(rest)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = cmd.spawn()?;
    let mut child = child;
    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    Ok(ChildProcess { child, stdin, stdout })
}

impl crate::execute::version::OsSystem {
    pub fn since_start(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    pub fn error_writer(&self) -> Box<dyn Write + Send> {
        Box::new(std::io::stderr())
    }

    pub fn spawn(
        &self,
        command: &[String],
        dir: &str,
        stderr: Box<dyn Write + Send>,
    ) -> std::io::Result<ChildProcess> {
        spawn_process(command, dir, stderr)
    }
}

pub fn new_system() -> crate::execute::version::OsSystem {
    crate::execute::version::OsSystem::new()
}
