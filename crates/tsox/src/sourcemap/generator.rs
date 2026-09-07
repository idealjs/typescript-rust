use super::mapping::{NameIndex, RawSourceMap, SourceIndex};
use std::collections::HashMap;

use crate::tspath::{self, ComparePathsOptions};

pub struct Generator {
    pub(super) path_options: ComparePathsOptions,
    pub(super) file: String,
    pub(super) source_root: String,
    pub(super) sources_directory_path: String,
    pub(super) raw_sources: Vec<String>,
    pub(super) sources: Vec<String>,
    pub(super) source_to_source_index_map: HashMap<String, SourceIndex>,
    pub(super) sources_content: Vec<Option<String>>,
    pub(super) names: Vec<String>,
    pub(super) name_to_name_index_map: HashMap<String, NameIndex>,
    pub(super) mappings: String,
    pub(super) last_generated_line: i32,
    pub(super) last_generated_character: i32,
    pub(super) last_source_index: SourceIndex,
    pub(super) last_source_line: i32,
    pub(super) last_source_character: i32,
    pub(super) last_name_index: NameIndex,
    pub(super) has_last: bool,
    pub(super) pending_generated_line: i32,
    pub(super) pending_generated_character: i32,
    pub(super) pending_source_index: SourceIndex,
    pub(super) pending_source_line: i32,
    pub(super) pending_source_character: i32,
    pub(super) pending_name_index: NameIndex,
    pub(super) has_pending: bool,
    pub(super) has_pending_source: bool,
    pub(super) has_pending_name: bool,
}

impl Generator {
    pub fn new(
        file: &str,
        source_root: &str,
        sources_directory_path: &str,
        options: ComparePathsOptions,
    ) -> Self {
        Generator {
            path_options: options,
            file: file.to_string(),
            source_root: source_root.to_string(),
            sources_directory_path: sources_directory_path.to_string(),
            raw_sources: Vec::new(),
            sources: Vec::new(),
            source_to_source_index_map: HashMap::new(),
            sources_content: Vec::new(),
            names: Vec::new(),
            name_to_name_index_map: HashMap::new(),
            mappings: String::new(),
            last_generated_line: 0,
            last_generated_character: 0,
            last_source_index: 0,
            last_source_line: 0,
            last_source_character: 0,
            last_name_index: 0,
            has_last: false,
            pending_generated_line: 0,
            pending_generated_character: 0,
            pending_source_index: 0,
            pending_source_line: 0,
            pending_source_character: 0,
            pending_name_index: 0,
            has_pending: false,
            has_pending_source: false,
            has_pending_name: false,
        }
    }

    pub fn sources(&self) -> &[String] {
        &self.raw_sources
    }

    pub fn add_source(&mut self, file_name: &str) -> SourceIndex {
        let source = tspath::get_relative_path_to_directory_or_url(
            &self.sources_directory_path,
            file_name,
            true,
            &self.path_options,
        );
        if let Some(&idx) = self.source_to_source_index_map.get(&source) {
            return idx;
        }
        let idx = self.sources.len() as SourceIndex;
        self.sources.push(source.clone());
        self.raw_sources.push(file_name.to_string());
        self.source_to_source_index_map.insert(source, idx);
        idx
    }

    pub fn set_source_content(
        &mut self,
        source_index: SourceIndex,
        content: &str,
    ) -> Result<(), String> {
        if source_index < 0 || source_index as usize >= self.sources.len() {
            return Err("sourceIndex is out of range".to_string());
        }
        let idx = source_index as usize;
        if self.sources_content.len() <= idx {
            self.sources_content.resize(idx + 1, None);
        }
        self.sources_content[idx] = Some(content.to_string());
        Ok(())
    }

    pub fn add_name(&mut self, name: &str) -> NameIndex {
        if let Some(&idx) = self.name_to_name_index_map.get(name) {
            return idx;
        }
        let idx = self.names.len() as NameIndex;
        self.names.push(name.to_string());
        self.name_to_name_index_map.insert(name.to_string(), idx);
        idx
    }

    pub fn raw_source_map(&mut self) -> RawSourceMap {
        self.commit_pending_mapping();
        RawSourceMap {
            version: 3,
            file: self.file.clone(),
            source_root: self.source_root.clone(),
            sources: self.sources.clone(),
            names: self.names.clone(),
            mappings: self.mappings.clone(),
            sources_content: self.sources_content.clone(),
        }
    }

    pub fn to_json(&mut self) -> String {
        let map = self.raw_source_map();
        crate::json::marshal(&map).unwrap_or_default()
    }

    pub fn to_base64_data_url(&mut self) -> String {
        let json = self.to_json();
        use base64::{Engine as _, engine::general_purpose};
        let encoded = general_purpose::STANDARD.encode(json.as_bytes());
        format!("data:application/json;base64,{encoded}")
    }
}
