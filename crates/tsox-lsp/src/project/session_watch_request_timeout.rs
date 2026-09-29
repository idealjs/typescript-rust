#![allow(unused_imports)]

use crate::project::session::*;

pub const WATCH_REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

pub const IDLE_CACHE_CLEAN_DELAY: Duration = Duration::from_secs(30);

pub const PERFORMANCE_TELEMETRY_INTERVAL: Duration = Duration::from_secs(300);

pub struct SessionInit {
    pub options: SessionOptions,
    pub fs: Arc<dyn FS>,
    pub client: Option<Arc<dyn Client>>,
    pub parse_cache: Option<Arc<ParseCache>>,
    pub logger: Option<Arc<dyn Logger>>,
    pub spawner: Option<Arc<dyn tsox_compile::mig::m3l_cm_2::Spawner>>,
    pub content_mapper_logger: Option<tsox_compile::mig::m3l_cm_2::Logger>,
    pub content_mapped_parse_cache: Option<Arc<ParseCache>>,
    pub background_ctx: tsox_core::core::mig::context::Context,
    pub npm_executor: Option<Arc<dyn crate::project::ata_ata::NpmExecutor>>,
}

pub struct Session {
    pub options: SessionOptions,
    pub start_time: Instant,
    pub to_path: Box<dyn Fn(&str) -> Path + Send + Sync>,
    pub client: Option<Arc<dyn Client>>,
    pub logger: Option<Arc<dyn Logger>>,

    pub fs: Option<Arc<OverlayFS>>,
    pub parse_cache: Option<Arc<ParseCache>>,
    pub content_mapped_parse_cache: Option<Arc<ParseCache>>,
    pub extended_config_cache: Option<Arc<ExtendedConfigCache>>,
    pub program_counter: Option<Arc<ProgramCounter>>,
    pub background_queue: Arc<background::Queue>,

    pub background_ctx: tsox_core::core::mig::context::Context,

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

    pub content_mapper_host: Option<Arc<tsox_compile::mig::m3l_cm_3::HostImpl>>,
    pub content_mapper_timings: Mutex<tsox_compile::mig::m3l_cm::Timings>,
    pub initial_user_preferences: Mutex<UserPreferences>,
    pub typings_installer: Option<Arc<crate::project::ata_ata::TypingsInstaller>>,
    pub registered_content_mapper_extensions: Mutex<Vec<String>>,
    pub registered_content_mapper_snapshot_id: AtomicU64,
    pub npm_executor: Option<Arc<dyn crate::project::ata_ata::NpmExecutor>>,
}
