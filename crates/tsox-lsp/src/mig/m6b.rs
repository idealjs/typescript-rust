use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tsox_core::tspath::{
    get_canonical_file_name, get_directory_path, get_normalized_absolute_path,
};
use tsox_emit::sourcemap::{MappingsDecoder, RawSourceMap, MISSING_SOURCE};

pub type SourceIndex = i32;
pub type NameIndex = i32;

const MISSING_POSITION: i32 = -1;

pub struct EcmaLineInfo {
    text: String,
    line_starts: Vec<tsox_core::core::text::TextPos>,
}

pub fn create_ecma_line_info(
    text: String,
    line_starts: Vec<tsox_core::core::text::TextPos>,
) -> EcmaLineInfo { ::tsox_core::fntrace::enter("create_ecma_line_info"); 
    EcmaLineInfo { text, line_starts }
}

impl EcmaLineInfo {
    pub fn line_count(&self) -> usize { ::tsox_core::fntrace::enter("line_count"); 
        self.line_starts.len()
    }

    pub fn line_text(&self, line: usize) -> &str { ::tsox_core::fntrace::enter("line_text"); 
        let pos = self.line_starts[line] as usize;
        let end = if line + 1 < self.line_starts.len() {
            self.line_starts[line + 1] as usize
        } else {
            self.text.len()
        };
        &self.text[pos..end]
    }
}

pub trait Host {
    fn use_case_sensitive_file_names(&self) -> bool;
    fn get_ecma_line_info(&self, file_name: &str) -> Option<EcmaLineInfo>;
    fn read_file(&self, file_name: &str) -> Option<String>;
}

#[derive(Clone, PartialEq, Eq)]
pub struct MappedPosition {
    pub generated_position: i32,
    pub source_position: i32,
    pub source_index: SourceIndex,
    pub name_index: NameIndex,
}

pub type SourceMappedPosition = MappedPosition;

impl MappedPosition {
    pub fn is_source_mapped_position(&self) -> bool { ::tsox_core::fntrace::enter("is_source_mapped_position"); 
        self.source_index != MISSING_SOURCE && self.source_position != MISSING_POSITION
    }
}

pub struct DocumentPosition {
    pub file_name: String,
    pub pos: i32,
}

pub struct DocumentPositionMapper {
    use_case_sensitive_file_names: bool,
    source_file_absolute_paths: Vec<String>,
    source_to_source_index_map: HashMap<String, SourceIndex>,
    generated_absolute_file_path: String,
    generated_mappings: Vec<MappedPosition>,
    source_mappings: HashMap<SourceIndex, Vec<SourceMappedPosition>>,
}

pub fn create_document_position_mapper(
    host: &dyn Host,
    source_map: &RawSourceMap,
    map_path: &str,
) -> DocumentPositionMapper { ::tsox_core::fntrace::enter("create_document_position_mapper"); 
    let map_directory = get_directory_path(map_path);
    let source_root = if !source_map.source_root.is_empty() {
        get_normalized_absolute_path(&source_map.source_root, &map_directory)
    } else {
        map_directory.clone()
    };
    let generated_absolute_file_path =
        get_normalized_absolute_path(&source_map.file, &map_directory);
    let source_file_absolute_paths: Vec<String> = source_map
        .sources
        .iter()
        .map(|source| get_normalized_absolute_path(source, &source_root))
        .collect();
    let use_case_sensitive_file_names = host.use_case_sensitive_file_names();
    let mut source_to_source_index_map =
        HashMap::with_capacity(source_file_absolute_paths.len());
    for (i, source) in source_file_absolute_paths.iter().enumerate() {
        source_to_source_index_map.insert(
            get_canonical_file_name(source, use_case_sensitive_file_names),
            i as SourceIndex,
        );
    }

    let mut decoded_mappings: Vec<MappedPosition> = Vec::new();
    let (mappings, error) = MappingsDecoder::new(&source_map.mappings).collect_all();
    for mapping in &mappings {
        let generated_position = match host.get_ecma_line_info(&generated_absolute_file_path) {
            Some(line_info) => tsox_frontend::scanner::mig::w2::compute_position_of_line_and_utf16_character(
                &line_info.line_starts,
                mapping.generated_line as usize,
                mapping.generated_character as usize,
                &line_info.text,
                true,
            ) as i32,
            None => -1,
        };

        let source_position = if mapping.is_source_mapping() {
            match host.get_ecma_line_info(&source_file_absolute_paths[mapping.source_index as usize])
            {
                Some(line_info) => {
                    tsox_frontend::scanner::mig::w2::compute_position_of_line_and_utf16_character(
                        &line_info.line_starts,
                        mapping.source_line as usize,
                        mapping.source_character as usize,
                        &line_info.text,
                        true,
                    ) as i32
                }
                None => -1,
            }
        } else {
            -1
        };

        decoded_mappings.push(MappedPosition {
            generated_position,
            source_index: mapping.source_index,
            source_position,
            name_index: mapping.name_index,
        });
    }
    if error.is_some() {
        decoded_mappings = Vec::new();
    }

    let mut source_mappings: HashMap<SourceIndex, Vec<SourceMappedPosition>> = HashMap::new();
    for mapping in &decoded_mappings {
        if !mapping.is_source_mapped_position() {
            continue;
        }
        let list = source_mappings.entry(mapping.source_index).or_default();
        list.push(SourceMappedPosition {
            generated_position: mapping.generated_position,
            source_index: mapping.source_index,
            source_position: mapping.source_position,
            name_index: mapping.name_index,
        });
    }
    for list in source_mappings.values_mut() {
        list.sort_by_key(|m| m.source_position);
        *list = tsox_core::core::mig::m3j_2::deduplicate_sorted(list, |a, b| {
            a.generated_position == b.generated_position
                && a.source_index == b.source_index
                && a.source_position == b.source_position
        });
    }

    let mut generated_mappings = decoded_mappings;
    generated_mappings.sort_by_key(|m| m.generated_position);
    generated_mappings = tsox_core::core::mig::m3j_2::deduplicate_sorted(&generated_mappings, |a, b| {
        a.generated_position == b.generated_position
            && a.source_index == b.source_index
            && a.source_position == b.source_position
    });

    DocumentPositionMapper {
        use_case_sensitive_file_names,
        source_file_absolute_paths,
        source_to_source_index_map,
        generated_absolute_file_path,
        generated_mappings,
        source_mappings,
    }
}

impl DocumentPositionMapper {
    pub fn get_source_position(&self, loc: &DocumentPosition) -> Option<DocumentPosition> { ::tsox_core::fntrace::enter("get_source_position"); 
        if self.generated_mappings.is_empty() {
            return None;
        }
        let target_index = self
            .generated_mappings
            .partition_point(|m| m.generated_position < loc.pos);
        if target_index >= self.generated_mappings.len() {
            return None;
        }
        let mapping = &self.generated_mappings[target_index];
        if !mapping.is_source_mapped_position() {
            return None;
        }
        Some(DocumentPosition {
            file_name: self.source_file_absolute_paths[mapping.source_index as usize].clone(),
            pos: mapping.source_position,
        })
    }

    pub fn get_generated_position(&self, loc: &DocumentPosition) -> Option<DocumentPosition> { ::tsox_core::fntrace::enter("get_generated_position"); 
        let source_index = *self
            .source_to_source_index_map
            .get(&get_canonical_file_name(
                &loc.file_name,
                self.use_case_sensitive_file_names,
            ))?;
        let source_mappings = self.source_mappings.get(&source_index)?;
        let target_index = source_mappings.partition_point(|m| m.source_position < loc.pos);
        if target_index >= source_mappings.len() {
            return None;
        }
        let mapping = &source_mappings[target_index];
        if mapping.source_index != source_index {
            return None;
        }
        Some(DocumentPosition {
            file_name: self.generated_absolute_file_path.clone(),
            pos: mapping.generated_position,
        })
    }
}

pub fn get_document_position_mapper(
    host: &dyn Host,
    generated_file_name: &str,
) -> Option<DocumentPositionMapper> { ::tsox_core::fntrace::enter("get_document_position_mapper"); 
    let mut map_file_name = try_get_source_mapping_url_for_file(host, generated_file_name);
    if !map_file_name.is_empty() {
        if let Some((base64_object, matched)) = try_parse_base64_url(&map_file_name) {
            if matched {
                if !base64_object.is_empty() {
                    if let Ok(decoded) = base64_std_decode(&base64_object)
                    {
                        return convert_document_to_source_mapper(
                            host,
                            &String::from_utf8(decoded).ok()?,
                            generated_file_name,
                        );
                    }
                }
                map_file_name = String::new();
            }
        }
    }

    let generated_map_location = format!("{}.map", generated_file_name);
    let mut possible_map_locations: Vec<&str> = Vec::new();
    if !map_file_name.is_empty() {
        possible_map_locations.push(&map_file_name);
    }
    possible_map_locations.push(&generated_map_location);
    let generated_dir = get_directory_path(generated_file_name);
    for location in &possible_map_locations {
        let map_file_name = get_normalized_absolute_path(location, &generated_dir);
        if let Some(map_file_contents) = host.read_file(&map_file_name) {
            return convert_document_to_source_mapper(host, &map_file_contents, &map_file_name);
        }
    }
    None
}

pub fn convert_document_to_source_mapper(
    host: &dyn Host,
    contents: &str,
    map_file_name: &str,
) -> Option<DocumentPositionMapper> { ::tsox_core::fntrace::enter("convert_document_to_source_mapper"); 
    let source_map = try_parse_raw_source_map(contents)?;
    if source_map.sources.is_empty() || source_map.file.is_empty() || source_map.mappings.is_empty()
    {
        return None;
    }
    if source_map.sources_content.iter().any(|s| s.is_some()) {
        return None;
    }
    Some(create_document_position_mapper(host, &source_map, map_file_name))
}

pub fn try_parse_raw_source_map(contents: &str) -> Option<RawSourceMap> { ::tsox_core::fntrace::enter("try_parse_raw_source_map"); 
    let source_map: RawSourceMap = tsox_core::json::unmarshal(contents).ok()?;
    if source_map.version != 3 {
        return None;
    }
    Some(source_map)
}

fn try_get_source_mapping_url_for_file(host: &dyn Host, file_name: &str) -> String { ::tsox_core::fntrace::enter("try_get_source_mapping_url_for_file"); 
    match host.get_ecma_line_info(file_name) {
        Some(line_info) => {
            for index in (0..line_info.line_count()).rev() {
                let line = line_info.line_text(index);
                let line = line.trim_start();
                let line = line.trim_end_matches(|ch| tsox_core::stringutil::is_line_break(ch));
                if line.is_empty() {
                    continue;
                }
                let bytes = line.as_bytes();
                if bytes.len() < 4
                    || !line.starts_with("//")
                    || (bytes[2] != b'#' && bytes[2] != b'@')
                    || bytes[3] != b' '
                {
                    break;
                }
                if let Some(url) = line[4..].strip_prefix("sourceMappingURL=") {
                    return url.trim_end().to_string();
                }
            }
            String::new()
        }
        None => String::new(),
    }
}

pub fn try_parse_base64_url(url: &str) -> Option<(String, bool)> { ::tsox_core::fntrace::enter("try_parse_base64_url"); 
    let rest = match url.strip_prefix("data:") {
        Some(rest) => rest,
        None => return None,
    };
    let rest = match rest.strip_prefix("application/json;") {
        Some(rest) => rest,
        None => return Some((String::new(), false)),
    };
    let mut rest = rest;
    if let Some(after_charset) = rest.strip_prefix("charset=") {
        if after_charset.len() < 6 || !after_charset[..6].eq_ignore_ascii_case("utf-8;") {
            return Some((String::new(), false));
        }
        rest = &after_charset[6..];
    }
    let rest = match rest.strip_prefix("base64,") {
        Some(rest) => rest,
        None => return Some((String::new(), false)),
    };
    for ch in rest.chars() {
        if !(tsox_core::stringutil::is_ascii_letter(ch)
            || tsox_core::stringutil::is_digit(ch)
            || ch == '+'
            || ch == '/'
            || ch == '=')
        {
            return Some((String::new(), false));
        }
    }
    Some((rest.to_string(), true))
}

fn base64_std_char_value(b: u8) -> Result<u32, String> { ::tsox_core::fntrace::enter("base64_std_char_value"); 
    match b {
        b'A'..=b'Z' => Ok((b - b'A') as u32),
        b'a'..=b'z' => Ok((b - b'a') as u32 + 26),
        b'0'..=b'9' => Ok((b - b'0') as u32 + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err(format!("base64: invalid input byte {b:#04x}")),
    }
}

fn base64_std_decode(input: &str) -> Result<Vec<u8>, String> { ::tsox_core::fntrace::enter("base64_std_decode"); 
    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    let mut acc: u32 = 0;
    let mut nbits = 0usize;
    let mut padded = false;
    for &b in input.as_bytes() {
        if b == b'=' {
            padded = true;
            continue;
        }
        if padded {
            return Err("base64: data continues after padding".to_string());
        }
        acc = (acc << 6) | base64_std_char_value(b)?;
        nbits += 6;
        if nbits >= 8 {
            nbits -= 8;
            out.push((acc >> nbits) as u8);
        }
    }
    if !padded && nbits % 8 != 0 && nbits < 8 {
        return Err("base64: truncated input".to_string());
    }
    Ok(out)
}

fn find_workspace_root(start: &Path) -> Option<PathBuf> { ::tsox_core::fntrace::enter("find_workspace_root"); 
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("Cargo.toml").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub fn test_data_path() -> PathBuf { ::tsox_core::fntrace::enter("test_data_path"); 
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = find_workspace_root(&manifest_dir)
        .unwrap_or_else(|| panic!("could not find Cargo.toml above {}", manifest_dir.display()));
    root.join("testdata")
}
