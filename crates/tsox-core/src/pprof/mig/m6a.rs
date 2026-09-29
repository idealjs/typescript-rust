use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct ProfileSession {
    cpu_file_path: PathBuf,
    mem_file_path: PathBuf,
    cpu_file: Option<File>,
    log_writer: Box<dyn Write + Send>,
}

pub fn begin_profiling(profile_dir: &str, log_writer: Box<dyn Write + Send>) -> ProfileSession {
    std::fs::create_dir_all(profile_dir).expect("failed to create profile dir");

    let pid = std::process::id();
    let cpu_profile_path = Path::new(profile_dir).join(format!("{pid}-cpuprofile.pb.gz"));
    let mem_profile_path = Path::new(profile_dir).join(format!("{pid}-memprofile.pb.gz"));
    let cpu_file = File::create(&cpu_profile_path).expect("failed to create cpu profile file");

    runtime_pprof::start_cpu_profile(&cpu_file).expect("failed to start cpu profile");

    ProfileSession {
        cpu_file_path: cpu_profile_path,
        mem_file_path: mem_profile_path,
        cpu_file: Some(cpu_file),
        log_writer,
    }
}

impl ProfileSession {
    pub fn stop(&mut self) {
        runtime_pprof::stop_cpu_profile();
        self.cpu_file = None;

        if !self.mem_file_path.as_os_str().is_empty() {
            let mut mem_file = File::create(&self.mem_file_path).expect("failed to create mem profile file");
            runtime_pprof::lookup("allocs")
                .write_to(&mut mem_file)
                .expect("failed to write alloc profile");
            let _ = writeln!(self.log_writer, "Memory profile: {}", self.mem_file_path.display());
        }

        let _ = writeln!(self.log_writer, "CPU profile: {}", self.cpu_file_path.display());
    }
}

pub struct CpuProfiler {
    session: Mutex<Option<ProfileSession>>,
}

impl Default for CpuProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuProfiler {
    pub fn new() -> Self {
        CpuProfiler {
            session: Mutex::new(None),
        }
    }

    pub fn start_cpu_profile(&self, profile_dir: &str) -> Result<(), String> {
        let mut session = self.session.lock().unwrap();

        if session.is_some() {
            return Err("CPU profiling already in progress".to_string());
        }

        std::fs::create_dir_all(profile_dir)
            .map_err(|e| format!("failed to create profile directory: {e}"))?;

        let cpu_profile_path = Path::new(profile_dir).join(format!(
            "{}-{}-cpuprofile.pb.gz",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0)
        ));
        let cpu_file = File::create(&cpu_profile_path)
            .map_err(|e| format!("failed to create CPU profile file: {e}"))?;

        if let Err(e) = runtime_pprof::start_cpu_profile(&cpu_file) {
            return Err(format!("failed to start CPU profile: {e}"));
        }

        *session = Some(ProfileSession {
            cpu_file_path: cpu_profile_path,
            mem_file_path: PathBuf::new(),
            cpu_file: Some(cpu_file),
            log_writer: Box::new(std::io::sink()),
        });
        Ok(())
    }

    pub fn stop_cpu_profile(&self) -> Result<String, String> {
        let mut session = self.session.lock().unwrap();

        match session.take() {
            None => Err("CPU profiling not in progress".to_string()),
            Some(mut s) => {
                let file_path = s.cpu_file_path.to_string_lossy().into_owned();
                s.stop();
                Ok(file_path)
            }
        }
    }
}

pub fn save_heap_profile(profile_dir: &str) -> Result<String, String> {
    std::fs::create_dir_all(profile_dir)
        .map_err(|e| format!("failed to create profile directory: {e}"))?;

    let heap_profile_path = Path::new(profile_dir).join(format!(
        "{}-{}-heapprofile.pb.gz",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    ));
    let mut heap_file = File::create(&heap_profile_path)
        .map_err(|e| format!("failed to create heap profile file: {e}"))?;

    runtime_gc();
    if let Err(e) = runtime_pprof::lookup("heap").write_to(&mut heap_file) {
        let _ = std::fs::remove_file(&heap_profile_path);
        return Err(format!("failed to write heap profile: {e}"));
    }

    Ok(heap_profile_path.to_string_lossy().into_owned())
}

pub fn save_alloc_profile(profile_dir: &str) -> Result<String, String> {
    std::fs::create_dir_all(profile_dir)
        .map_err(|e| format!("failed to create profile directory: {e}"))?;

    let alloc_profile_path = Path::new(profile_dir).join(format!(
        "{}-{}-allocprofile.pb.gz",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    ));
    let mut alloc_file = File::create(&alloc_profile_path)
        .map_err(|e| format!("failed to create alloc profile file: {e}"))?;

    if let Err(e) = runtime_pprof::lookup("allocs").write_to(&mut alloc_file) {
        let _ = std::fs::remove_file(&alloc_profile_path);
        return Err(format!("failed to write alloc profile: {e}"));
    }

    Ok(alloc_profile_path.to_string_lossy().into_owned())
}

pub fn run_gc() {
    runtime_gc();
}

mod runtime_pprof {
    use std::fmt;
    use std::io::Write;

    #[derive(Debug)]
    pub struct Error;

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("runtime profiling is not supported")
        }
    }

    pub struct Profile;

    impl Profile {
        pub fn write_to<W: Write + ?Sized>(&self, _w: &mut W) -> Result<(), Error> {
            Ok(())
        }
    }

    pub fn start_cpu_profile(_w: &dyn Write) -> Result<(), Error> {
        Ok(())
    }

    pub fn stop_cpu_profile() {}

    pub fn lookup(_name: &str) -> Profile {
        Profile
    }
}

fn runtime_gc() {
    }
