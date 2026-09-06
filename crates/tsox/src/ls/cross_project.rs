#![allow(dead_code)]

use std::sync::Arc;

use crate::compiler;
use crate::lsp::lsproto::lsp::{DocumentUri, Location, Position};
use crate::tspath;

use super::language_service::LanguageService;

pub trait Project: Send + Sync {
    fn id(&self) -> tspath::Path;
    fn get_program(&self) -> Option<Arc<compiler::Program>>;
    fn has_file(&self, file_name: &str) -> bool;
}

pub(crate) struct ProjectAndTextDocumentPosition {
    pub project: Arc<dyn Project>,
    pub ls: Option<Arc<LanguageService>>,
    pub uri: DocumentUri,
    pub position: Position,
    pub for_original_location: bool,
}

pub(crate) struct Response<Resp> {
    pub complete: bool,
    pub result: Resp,
    pub for_original_location: bool,
}

pub trait CrossProjectOrchestrator: Send + Sync {
    fn get_default_project(&self) -> Arc<dyn Project>;
    fn get_all_projects_for_initial_request(&self) -> Vec<Arc<dyn Project>>;
    fn get_language_service_for_project_with_file(
        &self,
        project: &dyn Project,
        uri: &DocumentUri,
    ) -> Option<Arc<LanguageService>>;
    fn get_projects_for_file(&self, uri: &DocumentUri) -> Result<Vec<Arc<dyn Project>>, String>;
    fn get_projects_loading_project_tree(
        &self,
        requested_project_trees: &tspath::Path,
    ) -> Vec<Arc<dyn Project>>;
}

pub fn combine_location_array<T: HasLocation>(
    combined: &mut Vec<T>,
    locations: &[T],
    seen: &mut std::collections::HashSet<String>,
) {
    for loc in locations {
        let l = loc.get_location();
        let key = format!(
            "{}:{}:{}",
            l.uri.0, l.range.start.line, l.range.start.character
        );
        if seen.insert(key) {
            combined.push(loc.clone());
        }
    }
}

pub trait HasLocation: Clone {
    fn get_location(&self) -> &Location;
}

pub trait HasLocations {
    fn get_locations(&self) -> Option<&Vec<Location>>;
}

impl HasLocation for Location {
    fn get_location(&self) -> &Location {
        self
    }
}

pub fn combine_response_locations<T: HasLocations>(results: &[T]) -> Option<Vec<Location>> {
    let mut combined: Vec<Location> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for resp in results {
        if let Some(locations) = resp.get_locations() {
            for loc in locations {
                let key = format!(
                    "{}:{}:{}",
                    loc.uri.0, loc.range.start.line, loc.range.start.character
                );
                if seen.insert(key) {
                    combined.push(loc.clone());
                }
            }
        }
    }
    if combined.is_empty() {
        None
    } else {
        Some(combined)
    }
}

pub fn combine_rename_edits(
    edits: &[std::collections::HashMap<DocumentUri, Vec<crate::lsp::lsproto::lsp::TextEdit>>],
) -> std::collections::HashMap<DocumentUri, Vec<crate::lsp::lsproto::lsp::TextEdit>> {
    let mut combined: std::collections::HashMap<
        DocumentUri,
        Vec<crate::lsp::lsproto::lsp::TextEdit>,
    > = std::collections::HashMap::new();
    let mut seen: std::collections::HashMap<DocumentUri, std::collections::HashSet<String>> =
        std::collections::HashMap::new();

    for edit_map in edits {
        for (uri, text_edits) in edit_map {
            let entry = combined.entry(uri.clone()).or_default();
            let seen_set = seen.entry(uri.clone()).or_default();
            for edit in text_edits {
                let key = format!(
                    "{}:{}:{}:{}:{}",
                    edit.range.start.line,
                    edit.range.start.character,
                    edit.range.end.line,
                    edit.range.end.character,
                    edit.new_text
                );
                if seen_set.insert(key) {
                    entry.push(edit.clone());
                }
            }
        }
    }
    combined
}

impl HasLocations for Vec<Location> {
    fn get_locations(&self) -> Option<&Vec<Location>> {
        Some(self)
    }
}

impl HasLocations for Option<Vec<Location>> {
    fn get_locations(&self) -> Option<&Vec<Location>> {
        self.as_ref()
    }
}
