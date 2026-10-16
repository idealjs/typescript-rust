#![allow(dead_code, unused_imports)]

use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, RwLock};

use tsox_core::tspath::Path;

use crate::project::session::*;

use crate::project::config_file_registry::ConfigFileRegistry;
use crate::project::snapshot::ResourceRequest;
use crate::project::snapshot_fs::SnapshotFS;
use crate::project::watch::{PatternsAndIgnored, WatchedFiles, get_recursive_glob_pattern};

pub(crate) use crate::project::ata_ata as ata;
pub(crate) use crate::project::logging_logger as logging_logger;

pub(crate) mod contentmapper {
    pub use tsox_compile::mig::m3l_cm::*;
    pub use tsox_compile::mig::m3l_cm_2::HostOptions;
    pub use tsox_compile::mig::m3l_cm_3::{HostImpl as Host, new_host_with_options};
}

pub(crate) mod background {
    pub use crate::project::background::*;
    pub use tsox_core::core::mig::context::Context;
}

pub(crate) mod lsproto {
    pub use crate::lsp::lsproto::*;
    pub use crate::mig::m5m::get_client_capabilities;
}

pub(crate) mod locale {
    pub use tsox_core::locale::*;
    pub use tsox_core::locale::mig::m6a::with_locale;
}

use super::m5e_6::new_snapshot;

#[derive(Clone, Default)]
pub(crate) struct ContentMapperContributions {
    pub mappers: Vec<Arc<contentmapper::Mapper>>,
    pub extensions: Vec<String>,
}

pub(crate) fn unsafe_clone_session(session: &Session) -> &'static Session { ::tsox_core::fntrace::enter("unsafe_clone_session"); 
    unsafe { &*(session as *const Session) }
}

struct SessionTypingsInstallerHost {
    npm_executor: Arc<dyn crate::project::ata_ata::NpmExecutor>,
    current_directory: String,
    fs: Arc<dyn tsox_tsoptions::vfs::FS>,
}

impl crate::project::ata_ata::NpmExecutor for SessionTypingsInstallerHost {
    fn npm_install(&self, cwd: &str, args: &[String]) -> Result<Vec<u8>, String> { ::tsox_core::fntrace::enter("npm_install"); 
        self.npm_executor.npm_install(cwd, args)
    }
}

impl crate::project::ata_ata::TypingsInstallerHost for SessionTypingsInstallerHost {
    fn get_current_directory(&self) -> &str { ::tsox_core::fntrace::enter("get_current_directory"); 
        &self.current_directory
    }
    fn fs(&self) -> &dyn tsox_tsoptions::vfs::FS { ::tsox_core::fntrace::enter("fs"); 
        self.fs.as_ref()
    }
}

pub fn new_content_mapper_host(init: &SessionInit) -> Option<Arc<contentmapper::Host>> { ::tsox_core::fntrace::enter("new_content_mapper_host"); 
    if !init.options.run_external_code || init.spawner.is_none() {
        return None;
    }
    let mut diagnostic_locale = locale::Locale::default_locale();
    if let Some(client) = &init.client {
        diagnostic_locale = client.get_locale();
    }
    Some(contentmapper::new_host_with_options(
        init.spawner.as_ref().unwrap().clone(),
        diagnostic_locale,
        contentmapper::HostOptions {
            logger: init.content_mapper_logger.clone(),
        },
    ))
}

impl Session {
    pub fn new_session(init: SessionInit) -> Self { ::tsox_core::fntrace::enter("new_session"); 
        let current_directory = init.options.current_directory.clone();
        let use_case_sensitive = init.fs.use_case_sensitive_file_names();
        let to_path: Box<dyn Fn(&str) -> Path + Send + Sync> =
            Box::new(move |file_name: &str| {
                tsox_core::tspath::to_path(file_name, &current_directory, use_case_sensitive)
            });
        let position_encoding = init.options.position_encoding.clone();
        let overlay_fs = Arc::new(OverlayFS::new(
            Arc::clone(&init.fs),
            HashMap::new(),
            position_encoding,
            {
                let cd = init.options.current_directory.clone();
                Box::new(move |f: &str| tsox_core::tspath::to_path(f, &cd, use_case_sensitive))
            },
        ));
        let parse_cache = init
            .parse_cache
            .clone()
            .unwrap_or_else(|| Arc::new(ParseCache::new(Default::default())));
        let content_mapped_parse_cache = init
            .content_mapped_parse_cache
            .clone()
            .unwrap_or_else(|| Arc::new(ParseCache::new(Default::default())));
        let extended_config_cache = Arc::new(ExtendedConfigCache::new());
        let session_logger = init
            .logger
            .clone()
            .unwrap_or_else(|| Arc::new(logging_logger::new_nop_logger()));
        let mut session = Session {
            options: init.options.clone(),
            start_time: Instant::now(),
            to_path,
            client: init.client.clone(),
            background_ctx: init.background_ctx.clone(),
            logger: Some(session_logger),
            fs: Some(overlay_fs),
            parse_cache: Some(parse_cache),
            content_mapped_parse_cache: Some(content_mapped_parse_cache),
            extended_config_cache: Some(extended_config_cache),
            program_counter: Some(Arc::new(ProgramCounter::new())),
            background_queue: Arc::new(background::Queue::new()),
            snapshot_id: std::sync::atomic::AtomicU64::new(0),
            snapshot: RwLock::new(None),
            pending_file_changes: Mutex::new(Vec::new()),
            pending_ata_changes: Mutex::new(HashMap::new()),
            watches: Arc::new(WatchRegistry::new()),
            seen_projects: Mutex::new(HashSet::new()),
            global_diag_publish_pending: std::sync::atomic::AtomicBool::new(false),
            workspace_user_preferences: Mutex::new(new_default_user_preferences()),
            pending_user_config_changes: std::sync::atomic::AtomicBool::new(false),
            scheduled_snapshot_update_generation: std::sync::atomic::AtomicU64::new(0),
            scheduled_snapshot_update_at: Mutex::new(None),
            diagnostics_refresh_generation: std::sync::atomic::AtomicU64::new(0),
            diagnostics_refresh_at: Mutex::new(None),
            idle_cache_clean_at: Mutex::new(None),
            warm_auto_import_active: std::sync::atomic::AtomicBool::new(false),
            content_mapper_host: new_content_mapper_host(&init),
            content_mapper_timings: Mutex::new(contentmapper::Timings::default()),
            initial_user_preferences: Mutex::new(new_default_user_preferences()),
            typings_installer: None,
            npm_executor: init.npm_executor.clone(),
            registered_content_mapper_extensions: Mutex::new(Vec::new()),
            registered_content_mapper_snapshot_id: std::sync::atomic::AtomicU64::new(0),
        };
        session.snapshot = RwLock::new(Some(Arc::new(new_snapshot(
            0,
            Arc::new(SnapshotFS::new(
                Arc::clone(&init.fs),
                HashMap::new(),
                {
                    let cd = init.options.current_directory.clone();
                    Box::new(move |f: &str| tsox_core::tspath::to_path(f, &cd, use_case_sensitive))
                },
            )),
            init.options.clone(),
            Box::new(ConfigFileRegistry::default()),
            None,
            new_default_user_preferences(),
            None,
            Some(Arc::new(WatchedFiles::new(
                "auto-import",
                lsproto::WATCH_KIND_CREATE | lsproto::WATCH_KIND_CHANGE | lsproto::WATCH_KIND_DELETE,
                {
                    let client_capabilities = crate::mig::m5m::ResolvedClientCapabilitiesContext {
                        capabilities: None,
                    };
                    lsproto::get_client_capabilities(&client_capabilities)
                        .raw
                        .get("workspace")
                        .and_then(|w| w.get("didChangeWatchedFiles"))
                        .and_then(|d| d.get("relativePatternSupport"))
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false)
                },
                move |node_modules_dirs: &HashMap<Path, String>| {
                    let mut patterns: Vec<String> = node_modules_dirs
                        .values()
                        .map(|dir| get_recursive_glob_pattern(dir))
                        .collect();
                    patterns.sort();
                    PatternsAndIgnored {
                        directories_outside_workspace: Vec::new(),
                        patterns_inside_workspace: patterns,
                        ignored: HashSet::new(),
                    }
                },
            ))),
        ))));
        if !init.options.typings_location.is_empty() && init.npm_executor.is_some() {
            session.typings_installer = Some(Arc::new(ata::TypingsInstaller::new(
                &ata::TypingsInstallerOptions {
                    typings_location: init.options.typings_location.clone(),
                    throttle_limit: 5,
                },
                Arc::new(SessionTypingsInstallerHost {
                    npm_executor: Arc::clone(init.npm_executor.as_ref().unwrap()),
                    current_directory: init.options.current_directory.clone(),
                    fs: Arc::clone(&init.fs),
                }),
            )));
        }
        if let Some(host) = &session.content_mapper_host {
            *session.content_mapper_timings.lock().unwrap() = host.timings();
        }
        session
    }

    pub fn background_context(&self) -> background::Context { ::tsox_core::fntrace::enter("background_context"); 
        self.with_current_locale(self.background_ctx())
    }

    pub fn background_ctx(&self) -> background::Context { ::tsox_core::fntrace::enter("background_ctx"); 
        self.background_ctx.clone()
    }

    pub fn with_current_locale(&self, ctx: background::Context) -> background::Context { ::tsox_core::fntrace::enter("with_current_locale"); 
        match &self.client {
            None => ctx,
            Some(client) => locale::with_locale(ctx, client.get_locale()),
        }
    }

    pub fn trace(&self, _msg: &str) { ::tsox_core::fntrace::enter("trace"); 
        panic!("ATA module resolution should not use tracing");
    }

    pub fn set_content_mapper_contributions(
        &self,
        contributions: ContentMapperContributions,
        document_uris: Vec<lsproto::DocumentUri>,
    ) { ::tsox_core::fntrace::enter("set_content_mapper_contributions"); 
        if !self.options.run_external_code {
            return;
        }
        self.cancel_scheduled_snapshot_update();
        let mut pending = self.pending_file_changes.lock().unwrap();
        let (changes, overlays) = self.flush_changes_locked();
        drop(pending);
        self.update_snapshot(
            overlays,
            SnapshotChange {
                reason: UpdateReason::DidChangeContentMapperContributions,
                file_changes: changes,
                content_mapper_contributions: Some(contributions),
                resource_request: crate::project::snapshot::ResourceRequest {
                    configured_project_documents: document_uris,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        if let Some(snapshot) = self.snapshot() {
            let _ = self.update_content_mapper_registrations(&snapshot);
        }
    }

    pub fn is_content_mapper_file(&self, uri: &lsproto::DocumentUri) -> bool { ::tsox_core::fntrace::enter("is_content_mapper_file"); 
        let snapshot = self.snapshot().expect("snapshot");
        let configured = snapshot.config_file_registry_content_mappers();
        let mut extensions = configured.extensions.clone();
        extensions.extend(snapshot.inferred_project_content_mapper_extensions.iter().cloned());
        let extension_strs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        tsox_core::tspath::file_extension_is_one_of(&uri.file_name(), &extension_strs)
    }
}
