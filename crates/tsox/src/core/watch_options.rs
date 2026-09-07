use crate::core::tristate::Tristate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum WatchFileKind {
    #[default]
    None = 0,
    FixedPollingInterval = 1,
    PriorityPollingInterval = 2,
    DynamicPriorityPolling = 3,
    FixedChunkSizePolling = 4,
    UseFsEvents = 5,
    UseFsEventsOnParentDirectory = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum WatchDirectoryKind {
    #[default]
    None = 0,
    UseFsEvents = 1,
    FixedPollingInterval = 2,
    DynamicPriorityPolling = 3,
    FixedChunkSizePolling = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum PollingKind {
    #[default]
    None = 0,
    FixedInterval = 1,
    PriorityInterval = 2,
    DynamicPriority = 3,
    FixedChunkSize = 4,
}

pub fn parse_watch_file_kind(s: &str) -> Option<WatchFileKind> {
    match s.to_lowercase().as_str() {
        "fixedpollinginterval" => Some(WatchFileKind::FixedPollingInterval),
        "prioritypollinginterval" => Some(WatchFileKind::PriorityPollingInterval),
        "dynamicprioritypolling" => Some(WatchFileKind::DynamicPriorityPolling),
        "fixedchunksizepolling" => Some(WatchFileKind::FixedChunkSizePolling),
        "usefsevents" => Some(WatchFileKind::UseFsEvents),
        "usefseventsonparentdirectory" => Some(WatchFileKind::UseFsEventsOnParentDirectory),
        _ => None,
    }
}

pub fn parse_watch_directory_kind(s: &str) -> Option<WatchDirectoryKind> {
    match s.to_lowercase().as_str() {
        "usefsevents" => Some(WatchDirectoryKind::UseFsEvents),
        "fixedpollinginterval" => Some(WatchDirectoryKind::FixedPollingInterval),
        "dynamicprioritypolling" => Some(WatchDirectoryKind::DynamicPriorityPolling),
        "fixedchunksizepolling" => Some(WatchDirectoryKind::FixedChunkSizePolling),
        _ => None,
    }
}

pub fn parse_polling_kind(s: &str) -> Option<PollingKind> {
    match s.to_lowercase().as_str() {
        "fixedinterval" => Some(PollingKind::FixedInterval),
        "priorityinterval" => Some(PollingKind::PriorityInterval),
        "dynamicpriority" => Some(PollingKind::DynamicPriority),
        "fixedchunksize" => Some(PollingKind::FixedChunkSize),
        _ => None,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WatchOptions {
    pub interval: Option<i32>,

    pub file_kind: WatchFileKind,

    pub directory_kind: WatchDirectoryKind,

    pub fallback_polling: PollingKind,

    pub sync_watch_dir: Tristate,

    pub exclude_dir: Vec<String>,

    pub exclude_files: Vec<String>,
}

impl WatchOptions {
    pub fn is_empty(&self) -> bool {
        self.interval.is_none()
            && self.file_kind == WatchFileKind::None
            && self.directory_kind == WatchDirectoryKind::None
            && self.fallback_polling == PollingKind::None
            && self.sync_watch_dir.is_unknown()
            && self.exclude_dir.is_empty()
            && self.exclude_files.is_empty()
    }

    pub fn watch_interval_ms(&self) -> i32 {
        self.interval.unwrap_or(2000)
    }
}

#[cfg(test)]
mod tests;
