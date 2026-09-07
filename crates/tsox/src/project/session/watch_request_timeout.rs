#![allow(unused_imports)]

use super::*;

pub const WATCH_REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

pub const IDLE_CACHE_CLEAN_DELAY: Duration = Duration::from_secs(30);

pub const PERFORMANCE_TELEMETRY_INTERVAL: Duration = Duration::from_secs(300);

pub struct SessionInit {
    pub options: SessionOptions,
    pub fs: Arc<dyn FS>,
    pub client: Option<Arc<dyn Client>>,
    pub parse_cache: Option<Arc<ParseCache>>,
    pub logger: Option<Arc<dyn Logger>>,
}

pub struct Session {
    pub options: SessionOptions,
    pub start_time: Instant,
    pub to_path: Box<dyn Fn(&str) -> Path + Send + Sync>,
    pub client: Option<Arc<dyn Client>>,
    pub logger: Option<Arc<dyn Logger>>,

    pub fs: Option<Arc<OverlayFS>>,
    pub parse_cache: Option<Arc<ParseCache>>,
    pub extended_config_cache: Option<Arc<ExtendedConfigCache>>,
    pub program_counter: Option<Arc<ProgramCounter>>,
    pub background_queue: Arc<background::Queue>,

    pub snapshot_id: AtomicU64,

    pub(crate) snapshot: RwLock<Option<Arc<Snapshot>>>,

    pub pending_file_changes: Mutex<Vec<FileChange>>,
    pub pending_ata_changes: Mutex<HashMap<Path, crate::project::snapshot::ATAStateChange>>,

    pub watches: Arc<WatchRegistry>,
    pub seen_projects: Mutex<HashSet<Path>>,
    pub global_diag_publish_pending: AtomicBool,

    pub(crate) workspace_user_preferences: Mutex<UserPreferences>,

    pub(crate) pending_user_config_changes: AtomicBool,

    pub(crate) scheduled_snapshot_update_generation: AtomicU64,
    pub(crate) scheduled_snapshot_update_at: Mutex<Option<Instant>>,

    pub(crate) diagnostics_refresh_generation: AtomicU64,
    pub(crate) diagnostics_refresh_at: Mutex<Option<Instant>>,

    pub(crate) idle_cache_clean_at: Mutex<Option<Instant>>,
    pub(crate) warm_auto_import_active: AtomicBool,
}
