#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::core::compiler_options::CompilerOptions;
use crate::lsp::lsproto;
use crate::tspath::Path;

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
    pub fn is_all_projects(&self) -> bool {
        self.referenced_projects.is_none()
    }

    pub fn is_project_referenced(&self, project_id: &Path) -> bool {
        self.referenced_projects
            .as_ref()
            .map(|s| s.contains(project_id))
            .unwrap_or(false)
    }

    pub fn projects(&self) -> Vec<Path> {
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
}

pub struct Snapshot {
    pub id: u64,
    pub parent_id: u64,
    ref_count: AtomicI32,

    pub fs: Option<Arc<super::snapshot_fs::SnapshotFS>>,
    pub project_collection: Option<Box<ProjectCollection>>,
    pub config_file_registry: Option<Box<ConfigFileRegistry>>,
    pub compiler_options_for_inferred_projects: Option<CompilerOptions>,
}

impl Snapshot {
    pub fn new(id: u64) -> Self {
        let s = Snapshot {
            id,
            parent_id: 0,
            ref_count: AtomicI32::new(0),
            fs: None,
            project_collection: None,
            config_file_registry: None,
            compiler_options_for_inferred_projects: None,
        };
        s.ref_count.store(1, Ordering::SeqCst);
        s
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn get_default_project(&self, _uri: &lsproto::DocumentUri) -> Option<&Project> {
        todo!("Snapshot::get_default_project requires full integration")
    }

    pub fn use_case_sensitive_file_names(&self) -> bool {
        true
    }

    pub fn read_file(&self, _file_name: &str) -> Option<String> {
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

    pub fn try_ref(&self) -> bool {
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

    pub fn deref_snapshot(&self) {
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

    fn dispose(&self) {}
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
