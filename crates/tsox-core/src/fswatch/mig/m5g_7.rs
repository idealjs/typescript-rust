#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::{Arc, LazyLock};

use super::m5f_2::FswatchError;
use super::m5g_2::{
    is_fswatch_err, Watch, WatchCallback, WatchDirectoryRequest, WatchOption, Watcher,
    ERR_FILESYSTEM_UNSUPPORTED, ERR_NIL_CALLBACK,
};
use super::m5g_5::WatcherImpl;

pub struct FallbackWatcher {
    pub primary: Arc<dyn Watcher>,
    pub secondary: Arc<dyn Watcher>,
}

impl FallbackWatcher {
    pub fn new(primary: Arc<dyn Watcher>, secondary: Arc<dyn Watcher>) -> FallbackWatcher { crate::fntrace::enter("new"); 
        FallbackWatcher { primary, secondary }
    }
}

impl Watcher for FallbackWatcher {
    fn name(&self) -> &str { crate::fntrace::enter("name"); 
        self.primary.name()
    }

    fn available(&self) -> bool { crate::fntrace::enter("available"); 
        self.primary.available()
    }

    fn has_fast_recursive_backend(&self) -> bool { crate::fntrace::enter("has_fast_recursive_backend"); 
        self.primary.has_fast_recursive_backend()
    }

    fn watch_directory(
        &self,
        dir: &str,
        callback: WatchCallback,
        options: &[WatchOption],
    ) -> Result<Arc<dyn Watch>, FswatchError> { crate::fntrace::enter("watch_directory"); 
        let requests = [WatchDirectoryRequest {
            dir: dir.to_string(),
            callback: Some(callback),
            options: options.to_vec(),
        }];
        let watches = self.watch_directories(&requests)?;
        Ok(watches.into_iter().next().unwrap())
    }

    fn watch_directories(
        &self,
        requests: &[WatchDirectoryRequest],
    ) -> Result<Vec<Arc<dyn Watch>>, FswatchError> { crate::fntrace::enter("watch_directories"); 
        match self.primary.watch_directories(requests) {
            Ok(watches) => Ok(watches),
            Err(err) if is_fswatch_err(&err, ERR_FILESYSTEM_UNSUPPORTED) => {
                let mut watches: Vec<Arc<dyn Watch>> = Vec::with_capacity(requests.len());
                for request in requests {
                    let cb = match &request.callback {
                        Some(cb) => Arc::clone(cb),
                        None => return Err(ERR_NIL_CALLBACK.to_string()),
                    };
                    let result = match self
                        .primary
                        .watch_directory(&request.dir, Arc::clone(&cb), &request.options)
                    {
                        Ok(w) => Ok(w),
                        Err(e) if is_fswatch_err(&e, ERR_FILESYSTEM_UNSUPPORTED) => self
                            .secondary
                            .watch_directory(&request.dir, cb, &request.options),
                        Err(e) => Err(e),
                    };
                    match result {
                        Ok(w) => watches.push(w),
                        Err(e) => {
                            for w in watches.iter().rev() {
                                let _ = w.close();
                            }
                            return Err(format!(
                                "fswatch: failed to watch directory \"{}\": {}",
                                request.dir, e
                            ));
                        }
                    }
                }
                Ok(watches)
            }
            Err(err) => Err(err),
        }
    }

    fn watch_file(
        &self,
        path: &str,
        callback: WatchCallback,
    ) -> Result<Arc<dyn Watch>, FswatchError> { crate::fntrace::enter("watch_file"); 
        match self.primary.watch_file(path, Arc::clone(&callback)) {
            Ok(w) => Ok(w),
            Err(err) if is_fswatch_err(&err, ERR_FILESYSTEM_UNSUPPORTED) => {
                self.secondary.watch_file(path, callback)
            }
            Err(err) => Err(err),
        }
    }
}

static INOTIFY_WATCHER: LazyLock<Arc<WatcherImpl>> =
    LazyLock::new(|| Arc::new(WatcherImpl::new("inotify")));
static FSEVENTS_WATCHER: LazyLock<Arc<WatcherImpl>> =
    LazyLock::new(|| Arc::new(WatcherImpl::new("fsevents")));
static KQUEUE_WATCHER: LazyLock<Arc<WatcherImpl>> =
    LazyLock::new(|| Arc::new(WatcherImpl::new("kqueue")));
static WINDOWS_WATCHER: LazyLock<Arc<WatcherImpl>> =
    LazyLock::new(|| Arc::new(WatcherImpl::new("windows")));
static FANOTIFY_WATCHER: LazyLock<Arc<WatcherImpl>> =
    LazyLock::new(|| Arc::new(WatcherImpl::new("fanotify")));
static FANOTIFY_FALLBACK_WATCHER: LazyLock<Arc<FallbackWatcher>> =
    LazyLock::new(|| Arc::new(FallbackWatcher::new(fanotify(), inotify())));

pub fn inotify() -> Arc<dyn Watcher> { crate::fntrace::enter("inotify"); 
    Arc::clone(&*INOTIFY_WATCHER) as Arc<dyn Watcher>
}

pub fn fsevents() -> Arc<dyn Watcher> { crate::fntrace::enter("fsevents"); 
    Arc::clone(&*FSEVENTS_WATCHER) as Arc<dyn Watcher>
}

pub fn kqueue() -> Arc<dyn Watcher> { crate::fntrace::enter("kqueue"); 
    Arc::clone(&*KQUEUE_WATCHER) as Arc<dyn Watcher>
}

pub fn windows() -> Arc<dyn Watcher> { crate::fntrace::enter("windows"); 
    Arc::clone(&*WINDOWS_WATCHER) as Arc<dyn Watcher>
}

pub fn fanotify() -> Arc<dyn Watcher> { crate::fntrace::enter("fanotify"); 
    Arc::clone(&*FANOTIFY_FALLBACK_WATCHER) as Arc<dyn Watcher>
}

pub fn all_watchers() -> Vec<Arc<dyn Watcher>> { crate::fntrace::enter("all_watchers"); 
    vec![inotify(), fsevents(), kqueue(), windows(), fanotify()]
}
