use std::error::Error;
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;

use crate::core::semaphore::{LimitedSemaphore, SemaphoreGuard, UnlimitedSemaphore};
use crate::core::tristate::Tristate;

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypeAcquisition {
    pub enable: Tristate,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub disable_filename_based_type_acquisition: Tristate,
}

impl TypeAcquisition {
    pub fn equals(&self, other: &TypeAcquisition) -> bool {
        self.enable == other.enable
            && self.include == other.include
            && self.exclude == other.exclude
            && self.disable_filename_based_type_acquisition
                == other.disable_filename_based_type_acquisition
    }
}

pub const VERSION: &str = "7.1.0-dev";

pub fn version() -> &'static str {
    VERSION
}

pub fn version_major_minor() -> &'static str {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut seen_major = false;
        let idx = VERSION
            .find(|r| {
                if r == '.' {
                    if seen_major {
                        return true;
                    }
                    seen_major = true;
                }
                false
            })
            .unwrap_or_else(|| panic!("invalid version string: {VERSION}"));
        VERSION[..idx].to_string()
    })
}

pub type ThrottleError = Box<dyn Error + Send + Sync>;

pub struct ThrottleGroup {
    semaphore_tx: mpsc::SyncSender<()>,
    semaphore_rx: Arc<Mutex<mpsc::Receiver<()>>>,
    handles: Mutex<Vec<JoinHandle<Result<(), ThrottleError>>>>,
}

pub fn new_throttle_group(max_concurrency: usize) -> ThrottleGroup {
    let (tx, rx) = mpsc::sync_channel(max_concurrency);
    ThrottleGroup {
        semaphore_tx: tx,
        semaphore_rx: Arc::new(Mutex::new(rx)),
        handles: Mutex::new(Vec::new()),
    }
}

impl ThrottleGroup {
    pub fn go(
        &self,
        f: impl FnOnce() -> Result<(), ThrottleError> + Send + 'static,
    ) {
        let tx = self.semaphore_tx.clone();
        let rx = Arc::clone(&self.semaphore_rx);
        let handle = std::thread::spawn(move || {
            if tx.send(()).is_err() {
                return Ok(());
            }
            let result = f();
            let _ = rx.lock().unwrap().recv();
            result
        });
        self.handles.lock().unwrap().push(handle);
    }

    pub fn wait(&self) -> Result<(), ThrottleError> {
        let handles: Vec<_> = self.handles.lock().unwrap().drain(..).collect();
        let mut first_err = None;
        for handle in handles {
            match handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    if first_err.is_none() {
                        first_err = Some(e);
                    }
                }
                Err(_) => {
                    if first_err.is_none() {
                        first_err = Some(Box::new(std::io::Error::other(
                            "throttle group worker panicked",
                        )));
                    }
                }
            }
        }
        match first_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}
