#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use crate::ls::cross_project::{CrossProjectOrchestrator, Project, Response as CrossProjectResponse};
use crate::ls::find_all_references::{NonLocalDefinition, SymbolAndEntriesData, SymbolEntryTransformOptions};
use crate::ls::language_service::LanguageService;
use crate::ls::call_hierarchy::CallHierarchyIncomingCall;
use crate::mig::m5n::LspError;
use crate::lsp::lsproto_lsp::{DocumentUri, Location, Position};
use crate::mig::m5p_support::IncomingEntry;
use tsox_core::tspath::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum VSReferenceKind {
    Inactive,
    Comment,
    String,
    Read,
    Write,
    Reference,
    Name,
    Qualified,
    TypeArgument,
    TypeConstraint,
    BaseType,
    Constructor,
    Destructor,
    Import,
    Declaration,
    AddressOf,
    NotReference,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct VSReferenceItem {
    pub vs_id: i32,
    pub vs_definition_id: Option<i32>,
    pub vs_location: Location,
    pub vs_definition_text: String,
    pub vs_kind: VSReferenceKind,
    pub vs_project_name: String,
    pub vs_containing_type: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct VSReferencesResponse {
    pub vs_reference_items: Vec<VSReferenceItem>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CallHierarchyIncomingCallsResponse {
    pub call_hierarchy_incoming_calls: Option<Vec<CallHierarchyIncomingCall>>,
}

pub fn combine_v_s_references(results: &[VSReferencesResponse]) -> VSReferencesResponse { ::tsox_core::fntrace::enter("combine_v_s_references"); 
    let mut combined: Vec<VSReferenceItem> = Vec::new();
    let mut next_id: i32 = 0;
    for resp in results {
        let mut id_map: HashMap<i32, i32> = HashMap::new();
        for item in &resp.vs_reference_items {
            let old_id = item.vs_id;
            let new_id = next_id;
            id_map.insert(old_id, new_id);
            next_id += 1;
            let mut new_item = item.clone();
            new_item.vs_id = new_id;
            if let Some(def_id) = item.vs_definition_id {
                new_item.vs_definition_id = Some(id_map.get(&def_id).copied().unwrap_or(0));
            }
            combined.push(new_item);
        }
    }
    VSReferencesResponse {
        vs_reference_items: combined,
    }
}

pub fn combine_incoming_calls(results: &[CallHierarchyIncomingCallsResponse]) -> CallHierarchyIncomingCallsResponse { ::tsox_core::fntrace::enter("combine_incoming_calls"); 
    let mut combined: Vec<CallHierarchyIncomingCall> = Vec::new();
    let mut seen_calls: HashSet<String> = HashSet::new();
    for resp in results {
        if let Some(calls) = &resp.call_hierarchy_incoming_calls {
            for call in calls {
                let loc = &call.from;
                let key = format!("{}:{}:{}", loc.uri.0, loc.range.start.line, loc.range.start.character);
                if seen_calls.insert(key) {
                    combined.push(call.clone());
                }
            }
        }
    }
    CallHierarchyIncomingCallsResponse {
        call_hierarchy_incoming_calls: Some(combined),
    }
}

struct Wt2CrossProjectItem {
    project: Arc<dyn Project>,
    from_default_ls: bool,
    ls: Option<Arc<LanguageService>>,
    uri: DocumentUri,
    position: Position,
    for_original_location: bool,
}

pub fn handle_cross_project<Resp>(
    default_ls: &LanguageService,
    params: &IncomingEntry,
    orchestrator: &dyn CrossProjectOrchestrator,
    symbol_and_entries_to_resp: impl FnMut(&LanguageService, &IncomingEntry, &SymbolAndEntriesData, &SymbolEntryTransformOptions) -> Result<Resp, LspError>,
    combine_results: fn(&[Resp]) -> Resp,
    is_rename: bool,
    implementations: bool,
    options: SymbolEntryTransformOptions,
    default_project_data: Option<&SymbolAndEntriesData>,
) -> Result<Resp, LspError> { ::tsox_core::fntrace::enter("handle_cross_project"); 
    let mut symbol_and_entries_to_resp = symbol_and_entries_to_resp;

    let default_project = orchestrator.get_default_project();
    let all_projects = orchestrator.get_all_projects_for_initial_request();

    let mut claimed: HashSet<Path> = HashSet::new();
    let mut results: HashMap<Path, CrossProjectResponse<Resp>> = HashMap::new();
    let mut default_definition: Option<NonLocalDefinition> = None;
    let mut first_error: Option<LspError> = None;
    let mut worklist: VecDeque<Wt2CrossProjectItem> = VecDeque::new();

    let mut enqueue = |item: Wt2CrossProjectItem, claimed: &mut HashSet<Path>, worklist: &mut VecDeque<Wt2CrossProjectItem>| {
        if claimed.insert(item.project.id()) {
            worklist.push_back(item);
        }
    };

    enqueue(
        Wt2CrossProjectItem {
            project: default_project.clone(),
            from_default_ls: true,
            ls: None,
            uri: DocumentUri(params.text_document_uri()),
            position: params.text_document_position(),
            for_original_location: false,
        },
        &mut claimed,
        &mut worklist,
    );
    for project in &all_projects {
        if !Arc::ptr_eq(project, &default_project) {
            enqueue(
                Wt2CrossProjectItem {
                    project: project.clone(),
                    from_default_ls: false,
                    ls: None,
                    uri: DocumentUri(params.text_document_uri()),
                    position: params.text_document_position(),
                    for_original_location: false,
                },
                &mut claimed,
                &mut worklist,
            );
        }
    }

    loop {
        while let Some(item) = worklist.pop_front() {
            let project_id = item.project.id();

            let mut ls_arc: Option<Arc<LanguageService>> = None;
            let ls: &LanguageService = if item.from_default_ls {
                default_ls
            } else if let Some(a) = &item.ls {
                ls_arc = Some(a.clone());
                ls_arc.as_ref().unwrap().as_ref()
            } else {
                ls_arc = orchestrator.get_language_service_for_project_with_file(item.project.as_ref(), &item.uri);
                match &ls_arc {
                    Some(a) => a.as_ref(),
                    None => continue,
                }
            };

            let mut owned_data: Option<SymbolAndEntriesData> = None;
            let mut has_data = false;
            let empty_data = SymbolAndEntriesData {
                original_node: params.node.clone(),
                symbols_and_entries: Vec::new(),
            };
            let data: &SymbolAndEntriesData = if item.from_default_ls {
                match default_project_data {
                    Some(d) => {
                        has_data = true;
                        d
                    }
                    None => match default_ls.provide_symbols_and_entries(&item.uri, item.position.clone(), is_rename, implementations) {
                        Some(d) => {
                            has_data = true;
                            owned_data = Some(d);
                            owned_data.as_ref().unwrap()
                        }
                        None => &empty_data,
                    },
                }
            } else {
                match ls.provide_symbols_and_entries(&item.uri, item.position.clone(), is_rename, implementations) {
                    Some(d) => {
                        has_data = true;
                        owned_data = Some(d);
                        owned_data.as_ref().unwrap()
                    }
                    None => &empty_data,
                }
            };

            if has_data {
                for entry in &data.symbols_and_entries {
                    if Arc::ptr_eq(&item.project, &default_project) && default_definition.is_none() {
                        default_definition = ls.get_non_local_definition(entry);
                    }
                    ls.for_each_original_definition_location(entry, &mut |uri: DocumentUri, position: Position| {
                        let Ok(def_projects) = orchestrator.get_projects_for_file(&uri) else {
                            return;
                        };
                        for def_project in def_projects {
                            enqueue(
                                Wt2CrossProjectItem {
                                    project: def_project.clone(),
                                    from_default_ls: false,
                                    ls: None,
                                    uri: uri.clone(),
                                    position: position.clone(),
                                    for_original_location: true,
                                },
                                &mut claimed,
                                &mut worklist,
                            );
                        }
                    });
                }
            }

            match symbol_and_entries_to_resp(ls, params, data, &options) {
                Ok(result) => {
                    results.insert(
                        project_id,
                        CrossProjectResponse {
                            complete: true,
                            result,
                            for_original_location: item.for_original_location,
                        },
                    );
                }
                Err(e) => {
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                }
            }
        }

        if let Some(err) = first_error {
            return Err(err);
        }

        let mut has_more_work = false;
        if let Some(def) = &default_definition {
            let completed_keys: Vec<Path> = results.keys().cloned().collect();
            for key in completed_keys {
                for loaded_project in orchestrator.get_projects_loading_project_tree(&key) {
                    if claimed.contains(&loaded_project.id()) || loaded_project.get_program().is_none() {
                        continue;
                    }
                    let mut enqueue_loaded = |uri: &DocumentUri, position: &Position| {
                        enqueue(
                            Wt2CrossProjectItem {
                                project: loaded_project.clone(),
                                from_default_ls: false,
                                ls: None,
                                uri: uri.clone(),
                                position: position.clone(),
                                for_original_location: false,
                            },
                            &mut claimed,
                            &mut worklist,
                        );
                        has_more_work = true;
                    };
                    if loaded_project.has_file(&def.uri.file_name()) {
                        enqueue_loaded(&def.uri, &def.position);
                    } else if let Some(source_pos) = def.get_source_position() {
                        if loaded_project.has_file(&source_pos.uri.file_name()) {
                            enqueue_loaded(&source_pos.uri, &source_pos.position);
                        }
                    } else if let Some(generated_pos) = def.get_generated_position() {
                        if loaded_project.has_file(&generated_pos.uri.file_name()) {
                            enqueue_loaded(&generated_pos.uri, &generated_pos.position);
                        }
                    }
                }
            }
        }
        if !has_more_work {
            break;
        }
    }

    let stored_count = claimed.len();
    let mut taken: HashMap<Path, CrossProjectResponse<Resp>> = std::mem::take(&mut results);

    let mut ordered: Vec<Resp> = Vec::new();
    let mut seen_projects: HashSet<Path> = HashSet::new();

    let default_id = default_project.id();
    if let Some(r) = taken.remove(&default_id) {
        if r.complete {
            ordered.push(r.result);
        }
    }
    seen_projects.insert(default_id);

    for project in &all_projects {
        let pid = project.id();
        if seen_projects.insert(pid.clone()) {
            if let Some(r) = taken.remove(&pid) {
                if r.complete {
                    ordered.push(r.result);
                }
            }
        }
    }

    let rest: Vec<(Path, CrossProjectResponse<Resp>)> = taken.into_iter().collect();
    let mut deferred: Vec<(Path, CrossProjectResponse<Resp>)> = Vec::new();
    for (key, r) in rest {
        if !r.complete {
            continue;
        }
        if r.for_original_location {
            deferred.push((key, r));
            continue;
        }
        if seen_projects.insert(key) {
            ordered.push(r.result);
        }
    }
    for (key, r) in deferred {
        if seen_projects.insert(key) {
            ordered.push(r.result);
        }
    }

    if stored_count > 1 {
        Ok(combine_results(&ordered))
    } else {
        match ordered.into_iter().next() {
            Some(r) => Ok(r),
            None => Err(LspError::new(
                crate::mig::m5n::ErrorCode::InternalError,
                "no completed cross-project responses",
            )),
        }
    }
}
