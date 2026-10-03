#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};
use std::sync::atomic::{AtomicI32, Ordering};

use crate::lsp::lsproto;
use crate::ls::lsutil::{UserPreferences, new_default_user_preferences};
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::tspath::Path;

use super::config_file_registry::ConfigFileRegistry;
use super::file_change::FileChangeSummary;
use super::project::Project;
use super::project_collection::ProjectCollection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpdateReason {
    #[default]
    Unknown,
    DidOpenFile,
    DidCloseFile,
    DidChangeCompilerOptionsForInferredProjects,
    RequestedLanguageServicePendingChanges,
    RequestedLanguageServiceProjectNotLoaded,
    RequestedLanguageServiceForFileNotOpen,
    RequestedLanguageServiceProjectDirty,
    RequestedLoadProjectTree,
    RequestedLanguageServiceWithAutoImports,
    IdleCleanDiskCache,
    DidChangeContentMapperContributions,
}

#[derive(Default, Clone)]
pub struct ResourceRequest {
    pub documents: Vec<lsproto::DocumentUri>,
    pub configured_project_documents: Vec<lsproto::DocumentUri>,
    pub projects: Vec<Path>,
    pub project_tree: Option<ProjectTreeRequest>,
    pub auto_imports: lsproto::DocumentUri,
}

#[derive(Clone)]
pub struct ProjectTreeRequest {
    pub referenced_projects: Option<HashSet<Path>>,
}

impl ProjectTreeRequest {
    pub fn is_all_projects(&self) -> bool { ::tsox_core::fntrace::enter("is_all_projects"); 
        self.referenced_projects.is_none()
    }

    pub fn is_project_referenced(&self, project_id: &Path) -> bool { ::tsox_core::fntrace::enter("is_project_referenced"); 
        self.referenced_projects
            .as_ref()
            .map(|s| s.contains(project_id))
            .unwrap_or(false)
    }

    pub fn projects(&self) -> Vec<Path> { ::tsox_core::fntrace::enter("projects"); 
        self.referenced_projects
            .as_ref()
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[derive(Default, Clone)]
pub struct SnapshotChange {
    pub resource_request: ResourceRequest,
    pub reason: UpdateReason,
    pub file_changes: FileChangeSummary,
    pub compiler_options_for_inferred_projects: Option<CompilerOptions>,
    pub clean_disk_cache: bool,
    pub content_mapper_contributions: Option<super::mig::m5e::ContentMapperContributions>,
    pub new_config: Option<UserPreferences>,
    pub ata_changes: HashMap<Path, ATAStateChange>,
}

pub struct Snapshot {
    pub id: u64,
    pub parent_id: u64,
    pub(crate) ref_count: AtomicI32,

    pub fs: Option<Arc<super::snapshot_fs::SnapshotFS>>,
    pub project_collection: Option<Box<ProjectCollection>>,
    pub config_file_registry: Option<Box<ConfigFileRegistry>>,
    pub compiler_options_for_inferred_projects: Option<CompilerOptions>,
    pub user_preferences: UserPreferences,
    pub auto_imports: Option<Arc<crate::ls::autoimport_registry::Registry>>,
    pub auto_imports_watch: Option<Arc<crate::project::watch::WatchedFiles<HashMap<Path, String>>>>,
    pub converters: Option<crate::mig::m5u_conv::M5uConverters>,
    pub content_mapper_watch_state_once: OnceLock<(Vec<String>, HashSet<Path>)>,
    pub inferred_project_content_mappers: Vec<tsox_compile::mig::m3l_cm::Mapper>,
    pub inferred_project_content_mapper_extensions: Vec<String>,
    pub builder_logs: Option<Box<super::logging_log_tree::LogTree>>,
}

impl Snapshot {
    pub fn new(id: u64) -> Self { ::tsox_core::fntrace::enter("new"); 
        let s = Snapshot {
            id,
            parent_id: 0,
            ref_count: AtomicI32::new(0),
            fs: None,
            project_collection: None,
            config_file_registry: None,
            compiler_options_for_inferred_projects: None,
            user_preferences: new_default_user_preferences(),
            auto_imports: None,
            auto_imports_watch: None,
            converters: None,
            content_mapper_watch_state_once: OnceLock::new(),
            inferred_project_content_mappers: Vec::new(),
            inferred_project_content_mapper_extensions: Vec::new(),
            builder_logs: None,
        };
        s.ref_count.store(1, Ordering::SeqCst);
        s
    }

    pub fn id(&self) -> u64 { ::tsox_core::fntrace::enter("id"); 
        self.id
    }

    pub fn builder_logs_string(&self) -> String { ::tsox_core::fntrace::enter("builder_logs_string"); 
        self.builder_logs
            .as_deref()
            .map_or(String::new(), |l| l.to_string())
    }

    pub fn get_default_project(&self, _uri: &lsproto::DocumentUri) -> Option<&Project> { ::tsox_core::fntrace::enter("get_default_project"); 
        todo!("Snapshot::get_default_project requires full integration")
    }

    pub fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names"); 
        true
    }

    pub fn read_file(&self, _file_name: &str) -> Option<String> { ::tsox_core::fntrace::enter("read_file"); 
        todo!("Snapshot::read_file requires fs integration")
    }

    pub fn r#ref(&self) {
        let prev = self.ref_count.fetch_add(1, Ordering::SeqCst);
        if prev <= 0 {
            panic!(
                "snapshot {}: ref on disposed snapshot, parentId={}",
                self.id, self.parent_id
            );
        }
    }

    pub fn try_ref(&self) -> bool { ::tsox_core::fntrace::enter("try_ref"); 
        loop {
            let rc = self.ref_count.load(Ordering::SeqCst);
            if rc <= 0 {
                return false;
            }
            if self
                .ref_count
                .compare_exchange(rc, rc + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return true;
            }
        }
    }

    pub fn deref_snapshot(&self) { ::tsox_core::fntrace::enter("deref_snapshot"); 
        let rc = self.ref_count.fetch_sub(1, Ordering::SeqCst) - 1;
        if rc < 0 {
            panic!(
                "snapshot {}: ref count below zero, parentId={}",
                self.id, self.parent_id
            );
        }
        if rc == 0 {
            self.dispose();
        }
    }

    fn dispose(&self) { ::tsox_core::fntrace::enter("dispose"); }
}

#[derive(Default, Clone)]
pub struct APISnapshotRequest {
    pub open_projects: Option<HashSet<String>>,
    pub close_projects: Option<HashSet<Path>>,
    pub open_files: Option<HashSet<lsproto::DocumentUri>>,
    pub close_files: Option<HashSet<Path>>,
}

#[derive(Clone)]
pub struct ATAStateChange {
    pub project_id: Path,
    pub typings_files: Vec<String>,
    pub typings_files_to_watch: Vec<String>,
}
