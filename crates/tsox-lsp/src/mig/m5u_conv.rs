#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::{Diagnostic, SourceFile};
use tsox_frontend::ast::Node;

use crate::lsp::lsproto_lsp::{DocumentUri, Location, Position, Range};
use crate::ls::lsconv_converters::{file_name_to_document_uri, language_kind_to_script_kind, PositionEncodingKind};

pub type M5uPosition = Position;
pub type M5uRange = Range;

pub type SpanMapFeature = u32;
pub type SpanMapFidelity = u32;

pub const SPANMAP_FIDELITY_EXACT: SpanMapFidelity = 1;
pub const SPANMAP_FIDELITY_NONE: SpanMapFidelity = 2;
pub const SPANMAP_FIDELITY_SINGLE_SEGMENT: SpanMapFidelity = 3;

pub const SPANMAP_FEATURE_INLAY_HINTS: SpanMapFeature = 1;

pub trait M5uFidelityExt {
    fn is_exact(&self) -> bool;
    fn is_none(&self) -> bool;
    fn is_single_segment(&self) -> bool;
}

impl M5uFidelityExt for SpanMapFidelity {
    fn is_exact(&self) -> bool { ::tsox_core::fntrace::enter("is_exact"); 
        *self == SPANMAP_FIDELITY_EXACT
    }
    fn is_none(&self) -> bool { ::tsox_core::fntrace::enter("is_none"); 
        *self == SPANMAP_FIDELITY_NONE
    }
    fn is_single_segment(&self) -> bool { ::tsox_core::fntrace::enter("is_single_segment"); 
        *self == SPANMAP_FIDELITY_SINGLE_SEGMENT
    }
}

impl crate::ls::lsconv_converters::Script for OriginalTextScriptOr<'_> {
    fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        M5uScript::file_name(self)
    }
    fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        M5uScript::text(self)
    }
}

impl crate::ls::lsconv_converters::Script for SourceFileScriptView {
    fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        M5uScript::file_name(self)
    }
    fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        M5uScript::text(self)
    }
}

impl crate::ls::lsconv_converters::Script for OriginalTextScript {
    fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        M5uScript::file_name(self)
    }
    fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        M5uScript::text(self)
    }
}

impl crate::ls::lsconv_converters::Converters {
    pub fn to_lsp_range_for_feature(
        &self,
        script: &dyn M5uScript,
        text_range: tsox_core::core::text::TextRange,
        feature: SpanMapFeature,
    ) -> (M5uRange, SpanMapFidelity) { ::tsox_core::fntrace::enter("to_lsp_range_for_feature"); 
        let (script, text_range, fidelity) =
            virtual_range_to_original(script, text_range, Some(feature));
        (
            M5uRange {
                start: self.position_to_line_and_character(&script, text_range.pos()),
                end: self.position_to_line_and_character(&script, text_range.end()),
            },
            fidelity,
        )
    }

    pub fn to_lsp_position_for_feature(
        &self,
        script: &dyn M5uScript,
        position: usize,
        feature: SpanMapFeature,
    ) -> (M5uPosition, SpanMapFidelity) { ::tsox_core::fntrace::enter("to_lsp_position_for_feature"); 
        let (script, position, fidelity) =
            virtual_position_to_original(script, position, Some(feature));
        (self.position_to_line_and_character(&script, position), fidelity)
    }

    fn m5u_lsp_position_to_virtual(
        &self,
        script: &dyn M5uScript,
        position: &Position,
        feature: SpanMapFeature,
    ) -> Vec<M5uMappedPositionRaw> { ::tsox_core::fntrace::enter("m5u_lsp_position_to_virtual"); 
        let view = PlainScriptView {
            file_name: script.file_name().to_string(),
            text: script.text().to_string(),
        };
        match script.span_map() {
            None => vec![M5uMappedPositionRaw {
                position: self.line_and_character_to_position(&view, position),
                fidelity: SPANMAP_FIDELITY_EXACT,
            }],
            Some(spans) => {
                let original = original_text_script(
                    script.original_file_name().to_string(),
                    script.original_text().to_string(),
                );
                let orig_offset = self.line_and_character_to_position(&original, position);
                spans.original_to_virtual_positions(orig_offset, feature)
            }
        }
    }

    pub fn from_lsp_position_for_source_file_m5u(
        &self,
        file: &Arc<SourceFile>,
        position: Position,
        feature: SpanMapFeature,
    ) -> Vec<M5uFileMappedPosition> { ::tsox_core::fntrace::enter("from_lsp_position_for_source_file_m5u"); 
        let files = source_file_projections(file);
        let mut result = Vec::new();
        for script in &files {
            let script_view = SourceFileScriptView { file: Arc::clone(script) };
            for mapped in self.m5u_lsp_position_to_virtual(&script_view, &position, feature) {
                result.push(M5uFileMappedPosition {
                    file: Arc::clone(script),
                    position: mapped.position,
                    fidelity: mapped.fidelity,
                });
            }
        }
        result
    }
}

pub trait M5uSpanMap {
    fn original_to_virtual_spans(
        &self,
        range: tsox_core::core::text::TextRange,
        feature: SpanMapFeature,
    ) -> Vec<M5uMappedSpanRaw>;
    fn original_to_virtual_intersecting_spans(
        &self,
        range: tsox_core::core::text::TextRange,
        feature: SpanMapFeature,
    ) -> Vec<M5uMappedSpanRaw>;
    fn original_to_virtual_positions(
        &self,
        position: usize,
        feature: SpanMapFeature,
    ) -> Vec<M5uMappedPositionRaw>;
    fn virtual_to_original_span(
        &self,
        range: tsox_core::core::text::TextRange,
    ) -> (tsox_core::core::text::TextRange, SpanMapFidelity);
    fn virtual_to_original_span_for_feature(
        &self,
        range: tsox_core::core::text::TextRange,
        feature: SpanMapFeature,
    ) -> (tsox_core::core::text::TextRange, SpanMapFidelity);
    fn virtual_to_original_position(
        &self,
        position: usize,
    ) -> (usize, SpanMapFidelity);
    fn virtual_to_original_position_for_feature(
        &self,
        position: usize,
        feature: SpanMapFeature,
    ) -> (usize, SpanMapFidelity);
}

pub struct M5uMappedSpanRaw {
    pub span: tsox_core::core::text::TextRange,
    pub fidelity: SpanMapFidelity,
}

pub struct M5uMappedPositionRaw {
    pub position: usize,
    pub fidelity: SpanMapFidelity,
}

pub trait M5uScript {
    fn file_name(&self) -> &str;
    fn original_file_name(&self) -> &str;
    fn text(&self) -> &str;
    fn span_map(&self) -> Option<&dyn M5uSpanMap>;
    fn original_text(&self) -> &str;
}

pub struct M5uMappedSpan<'a, T: ?Sized> {
    pub script: &'a T,
    pub span: tsox_core::core::text::TextRange,
    pub fidelity: SpanMapFidelity,
}

pub struct M5uMappedPosition<'a, T: ?Sized> {
    pub script: &'a T,
    pub position: usize,
    pub fidelity: SpanMapFidelity,
}

pub struct M5uConverters {
    pub get_line_map: Box<dyn Fn(&str) -> crate::ls::lsconv_linemap::LspLineMap + Send + Sync>,
    pub position_encoding: PositionEncodingKind,
}

pub fn new_converters(
    position_encoding: PositionEncodingKind,
    get_line_map: Box<dyn Fn(&str) -> crate::ls::lsconv_linemap::LspLineMap + Send + Sync>,
) -> M5uConverters { ::tsox_core::fntrace::enter("new_converters"); 
    M5uConverters {
        get_line_map,
        position_encoding,
    }
}

pub fn original_text_script(file_name: String, text: String) -> OriginalTextScript { ::tsox_core::fntrace::enter("original_text_script"); 
    OriginalTextScript { file_name, text }
}

pub struct OriginalTextScript {
    pub file_name: String,
    pub text: String,
}

impl M5uScript for OriginalTextScript {
    fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        &self.file_name
    }

    fn original_file_name(&self) -> &str { ::tsox_core::fntrace::enter("original_file_name"); 
        &self.file_name
    }

    fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        &self.text
    }

    fn span_map(&self) -> Option<&dyn M5uSpanMap> { ::tsox_core::fntrace::enter("span_map"); 
        None
    }

    fn original_text(&self) -> &str { ::tsox_core::fntrace::enter("original_text"); 
        &self.text
    }
}

impl OriginalTextScript {
    pub fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        &self.file_name
    }

    pub fn original_file_name(&self) -> &str { ::tsox_core::fntrace::enter("original_file_name"); 
        &self.file_name
    }

    pub fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        &self.text
    }

    pub fn original_text(&self) -> &str { ::tsox_core::fntrace::enter("original_text"); 
        &self.text
    }

    pub fn span_map(&self) -> Option<&dyn M5uSpanMap> { ::tsox_core::fntrace::enter("span_map"); 
        None
    }
}

impl M5uConverters {
    pub fn to_lsp_range(
        &self,
        script: &dyn M5uScript,
        text_range: tsox_core::core::text::TextRange,
    ) -> (M5uRange, SpanMapFidelity) { ::tsox_core::fntrace::enter("to_lsp_range"); 
        let (script, text_range, fidelity) = virtual_range_to_original(script, text_range, None);
        (
            M5uRange {
                start: self.position_to_line_and_character(&script, text_range.pos()),
                end: self.position_to_line_and_character(&script, text_range.end()),
            },
            fidelity,
        )
    }

    pub fn to_lsp_range_for_feature(
        &self,
        script: &dyn M5uScript,
        text_range: tsox_core::core::text::TextRange,
        feature: SpanMapFeature,
    ) -> (M5uRange, SpanMapFidelity) { ::tsox_core::fntrace::enter("to_lsp_range_for_feature"); 
        let (script, text_range, fidelity) =
            virtual_range_to_original(script, text_range, Some(feature));
        (
            M5uRange {
                start: self.position_to_line_and_character(&script, text_range.pos()),
                end: self.position_to_line_and_character(&script, text_range.end()),
            },
            fidelity,
        )
    }

    pub fn to_lsp_position(
        &self,
        script: &dyn M5uScript,
        position: usize,
    ) -> (M5uPosition, SpanMapFidelity) { ::tsox_core::fntrace::enter("to_lsp_position"); 
        let (script, position, fidelity) = virtual_position_to_original(script, position, None);
        (self.position_to_line_and_character(&script, position), fidelity)
    }

    pub fn to_lsp_position_for_feature(
        &self,
        script: &dyn M5uScript,
        position: usize,
        feature: SpanMapFeature,
    ) -> (M5uPosition, SpanMapFidelity) { ::tsox_core::fntrace::enter("to_lsp_position_for_feature"); 
        let (script, position, fidelity) =
            virtual_position_to_original(script, position, Some(feature));
        (self.position_to_line_and_character(&script, position), fidelity)
    }

    pub fn to_lsp_location_for_feature(
        &self,
        script: &dyn M5uScript,
        rng: tsox_core::core::text::TextRange,
        feature: SpanMapFeature,
    ) -> (Location, SpanMapFidelity) { ::tsox_core::fntrace::enter("to_lsp_location_for_feature"); 
        let (lsp_range, fidelity) = self.to_lsp_range_for_feature(script, rng, feature);
        (
            Location {
                uri: DocumentUri(file_name_to_document_uri(script.original_file_name())),
                range: lsp_range,
            },
            fidelity,
        )
    }

    pub fn line_and_character_to_position(
        &self,
        script: &dyn M5uScript,
        line_and_character: &Position,
    ) -> usize { ::tsox_core::fntrace::enter("line_and_character_to_position"); 
        convention_line_and_character_to_position(self, script, line_and_character)
    }

    pub fn position_to_line_and_character(
        &self,
        script: &dyn M5uScript,
        position: usize,
    ) -> Position { ::tsox_core::fntrace::enter("position_to_line_and_character"); 
        convention_position_to_line_and_character(self, script, position)
    }

    fn lsp_range_to_virtual(
        &self,
        script: &dyn M5uScript,
        text_range: &M5uRange,
        feature: SpanMapFeature,
    ) -> Vec<M5uMappedSpanRaw> { ::tsox_core::fntrace::enter("lsp_range_to_virtual"); 
        match script.span_map() {
            None => vec![M5uMappedSpanRaw {
                span: tsox_core::core::text::TextRange::new(
                    self.line_and_character_to_position(script, &text_range.start),
                    self.line_and_character_to_position(script, &text_range.end),
                ),
                fidelity: SPANMAP_FIDELITY_EXACT,
            }],
            Some(spans) => {
                let original = original_text_script(
                    script.original_file_name().to_string(),
                    script.original_text().to_string(),
                );
                let orig_range = tsox_core::core::text::TextRange::new(
                    self.line_and_character_to_position(&original, &text_range.start),
                    self.line_and_character_to_position(&original, &text_range.end),
                );
                spans.original_to_virtual_spans(orig_range, feature)
            }
        }
    }

    fn lsp_position_to_virtual(
        &self,
        script: &dyn M5uScript,
        position: &Position,
        feature: SpanMapFeature,
    ) -> Vec<M5uMappedPositionRaw> { ::tsox_core::fntrace::enter("lsp_position_to_virtual"); 
        match script.span_map() {
            None => vec![M5uMappedPositionRaw {
                position: self.line_and_character_to_position(script, position),
                fidelity: SPANMAP_FIDELITY_EXACT,
            }],
            Some(spans) => {
                let original = original_text_script(
                    script.original_file_name().to_string(),
                    script.original_text().to_string(),
                );
                let orig_offset = self.line_and_character_to_position(&original, position);
                spans.original_to_virtual_positions(orig_offset, feature)
            }
        }
    }
}

pub fn from_lsp_range<'a>(
    c: &M5uConverters,
    script: &'a dyn M5uScript,
    text_range: M5uRange,
    feature: SpanMapFeature,
) -> Vec<M5uMappedSpan<'a, dyn M5uScript + 'a>> { ::tsox_core::fntrace::enter("from_lsp_range"); 
    lsp_range_to_virtual(c, std::slice::from_ref(&script), text_range, feature)
}

pub fn from_lsp_range_for_source_file(
    c: &M5uConverters,
    file: &Arc<SourceFile>,
    text_range: M5uRange,
    feature: SpanMapFeature,
) -> Vec<M5uFileMappedSpan> { ::tsox_core::fntrace::enter("from_lsp_range_for_source_file"); 
    let files = source_file_projections(file);
    lsp_range_to_virtual_for_files(c, &files, text_range, feature)
}

pub fn from_lsp_range_intersecting_for_source_file(
    c: &M5uConverters,
    file: &Arc<SourceFile>,
    text_range: M5uRange,
    feature: SpanMapFeature,
) -> Vec<M5uFileMappedSpan> { ::tsox_core::fntrace::enter("from_lsp_range_intersecting_for_source_file"); 
    let files = source_file_projections(file);
    let mut result = Vec::with_capacity(files.len());
    for script in &files {
        let script_view = SourceFileScriptView { file: Arc::clone(script) };
        match script_view.span_map_boxed() {
            None => {
                result.push(M5uFileMappedSpan {
                    file: Arc::clone(script),
                    span: tsox_core::core::text::TextRange::new(
                        c.line_and_character_to_position(&script_view, &text_range.start),
                        c.line_and_character_to_position(&script_view, &text_range.end),
                    ),
                    fidelity: SPANMAP_FIDELITY_EXACT,
                });
            }
            Some(spans) => {
                let original = original_text_script(
                    script_view.original_file_name().to_string(),
                    script_view.original_text().to_string(),
                );
                let original_range = tsox_core::core::text::TextRange::new(
                    c.line_and_character_to_position(&original, &text_range.start),
                    c.line_and_character_to_position(&original, &text_range.end),
                );
                for mapped in spans.original_to_virtual_intersecting_spans(original_range, feature)
                {
                    result.push(M5uFileMappedSpan {
                        file: Arc::clone(script),
                        span: mapped.span,
                        fidelity: mapped.fidelity,
                    });
                }
            }
        }
    }
    result
}

pub struct M5uFileMappedSpan {
    pub file: Arc<SourceFile>,
    pub span: tsox_core::core::text::TextRange,
    pub fidelity: SpanMapFidelity,
}

pub struct SourceFileScriptView {
    pub file: Arc<SourceFile>,
}

impl SourceFileScriptView {
    fn span_map_boxed(&self) -> Option<&'static dyn M5uSpanMap> { ::tsox_core::fntrace::enter("span_map_boxed"); 
        source_file_span_map(&self.file)
    }
}

impl M5uScript for SourceFileScriptView {
    fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        &self.file.file_name
    }

    fn original_file_name(&self) -> &str { ::tsox_core::fntrace::enter("original_file_name"); 
        source_file_original_file_name(&self.file)
    }

    fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        &self.file.text
    }

    fn span_map(&self) -> Option<&dyn M5uSpanMap> { ::tsox_core::fntrace::enter("span_map"); 
        source_file_span_map(&self.file)
    }

    fn original_text(&self) -> &str { ::tsox_core::fntrace::enter("original_text"); 
        source_file_original_text(&self.file)
    }
}

pub fn lsp_range_to_virtual<'a>(
    c: &M5uConverters,
    scripts: &[&'a dyn M5uScript],
    text_range: M5uRange,
    feature: SpanMapFeature,
) -> Vec<M5uMappedSpan<'a, dyn M5uScript + 'a>> { ::tsox_core::fntrace::enter("lsp_range_to_virtual"); 
    let mut result = Vec::with_capacity(scripts.len());
    for script in scripts {
        for mapped in c.lsp_range_to_virtual(*script, &text_range, feature) {
            result.push(M5uMappedSpan {
                script: *script,
                span: mapped.span,
                fidelity: mapped.fidelity,
            });
        }
    }
    result
}

pub fn lsp_range_to_virtual_for_files(
    c: &M5uConverters,
    files: &[Arc<SourceFile>],
    text_range: M5uRange,
    feature: SpanMapFeature,
) -> Vec<M5uFileMappedSpan> { ::tsox_core::fntrace::enter("lsp_range_to_virtual_for_files"); 
    let mut result = Vec::with_capacity(files.len());
    for file in files {
        let script = SourceFileScriptView { file: Arc::clone(file) };
        for mapped in c.lsp_range_to_virtual(&script, &text_range, feature) {
            result.push(M5uFileMappedSpan {
                file: Arc::clone(file),
                span: mapped.span,
                fidelity: mapped.fidelity,
            });
        }
    }
    result
}

pub fn from_lsp_position<'a>(
    c: &M5uConverters,
    script: &'a dyn M5uScript,
    position: Position,
    feature: SpanMapFeature,
) -> Vec<M5uMappedPosition<'a, dyn M5uScript + 'a>> { ::tsox_core::fntrace::enter("from_lsp_position"); 
    lsp_position_to_virtual(c, std::slice::from_ref(&script), position, feature)
}

pub fn from_lsp_position_for_source_file<'a>(
    c: &M5uConverters,
    file: &'a Arc<SourceFile>,
    position: Position,
    feature: SpanMapFeature,
) -> Vec<M5uFileMappedPosition> { ::tsox_core::fntrace::enter("from_lsp_position_for_source_file"); 
    let files = source_file_projections(file);
    let mut result = Vec::new();
    for script in &files {
        let script_view = SourceFileScriptView { file: Arc::clone(script) };
        for mapped in c.lsp_position_to_virtual(&script_view, &position, feature) {
            result.push(M5uFileMappedPosition {
                file: Arc::clone(script),
                position: mapped.position,
                fidelity: mapped.fidelity,
            });
        }
    }
    result
}

pub struct M5uFileMappedPosition {
    pub file: Arc<SourceFile>,
    pub position: usize,
    pub fidelity: SpanMapFidelity,
}

pub fn lsp_position_to_virtual<'a>(
    c: &M5uConverters,
    scripts: &[&'a dyn M5uScript],
    position: Position,
    feature: SpanMapFeature,
) -> Vec<M5uMappedPosition<'a, dyn M5uScript + 'a>> { ::tsox_core::fntrace::enter("lsp_position_to_virtual"); 
    let mut result = Vec::with_capacity(scripts.len());
    for script in scripts {
        for mapped in c.lsp_position_to_virtual(*script, &position, feature) {
            result.push(M5uMappedPosition {
                script: *script,
                position: mapped.position,
                fidelity: mapped.fidelity,
            });
        }
    }
    result
}

pub fn from_lsp_range_to_original(
    c: &M5uConverters,
    script: &dyn M5uScript,
    text_range: M5uRange,
) -> tsox_core::core::text::TextRange { ::tsox_core::fntrace::enter("from_lsp_range_to_original"); 
    let original = original_text_script(
        script.original_file_name().to_string(),
        script.original_text().to_string(),
    );
    tsox_core::core::text::TextRange::new(
        c.line_and_character_to_position(&original, &text_range.start),
        c.line_and_character_to_position(&original, &text_range.end),
    )
}

pub fn source_file_projections(file: &Arc<SourceFile>) -> Vec<Arc<SourceFile>> { ::tsox_core::fntrace::enter("source_file_projections"); 
    let supplemental = source_file_supplemental_source_files(file);
    let mut files = Vec::with_capacity(1 + supplemental.len());
    files.push(Arc::clone(file));
    files.extend(supplemental);
    files
}

pub fn virtual_range_to_original(
    script: &dyn M5uScript,
    text_range: tsox_core::core::text::TextRange,
    feature: Option<SpanMapFeature>,
) -> (OriginalTextScriptOr<'_>, tsox_core::core::text::TextRange, SpanMapFidelity) { ::tsox_core::fntrace::enter("virtual_range_to_original"); 
    match script.span_map() {
        None => (
            OriginalTextScriptOr::Script(script),
            text_range,
            SPANMAP_FIDELITY_EXACT,
        ),
        Some(spans) => {
            let (mapped, fidelity) = match feature {
                None => spans.virtual_to_original_span(text_range),
                Some(feature) => spans.virtual_to_original_span_for_feature(text_range, feature),
            };
            (
                OriginalTextScriptOr::Original(original_text_script(
                    script.original_file_name().to_string(),
                    script.original_text().to_string(),
                )),
                mapped,
                fidelity,
            )
        }
    }
}

pub enum OriginalTextScriptOr<'a> {
    Script(&'a dyn M5uScript),
    Original(OriginalTextScript),
}

impl M5uScript for OriginalTextScriptOr<'_> {
    fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        match self {
            OriginalTextScriptOr::Script(s) => s.file_name(),
            OriginalTextScriptOr::Original(s) => s.file_name(),
        }
    }

    fn original_file_name(&self) -> &str { ::tsox_core::fntrace::enter("original_file_name"); 
        match self {
            OriginalTextScriptOr::Script(s) => s.original_file_name(),
            OriginalTextScriptOr::Original(s) => s.original_file_name(),
        }
    }

    fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        match self {
            OriginalTextScriptOr::Script(s) => s.text(),
            OriginalTextScriptOr::Original(s) => s.text(),
        }
    }

    fn span_map(&self) -> Option<&dyn M5uSpanMap> { ::tsox_core::fntrace::enter("span_map"); 
        match self {
            OriginalTextScriptOr::Script(s) => s.span_map(),
            OriginalTextScriptOr::Original(s) => s.span_map(),
        }
    }

    fn original_text(&self) -> &str { ::tsox_core::fntrace::enter("original_text"); 
        match self {
            OriginalTextScriptOr::Script(s) => s.original_text(),
            OriginalTextScriptOr::Original(s) => s.original_text(),
        }
    }
}

pub fn virtual_position_to_original(
    script: &dyn M5uScript,
    position: usize,
    feature: Option<SpanMapFeature>,
) -> (OriginalTextScriptOr<'_>, usize, SpanMapFidelity) { ::tsox_core::fntrace::enter("virtual_position_to_original"); 
    match script.span_map() {
        None => (OriginalTextScriptOr::Script(script), position, SPANMAP_FIDELITY_EXACT),
        Some(spans) => {
            let (mapped, fidelity) = match feature {
                None => spans.virtual_to_original_position(position),
                Some(feature) => spans.virtual_to_original_position_for_feature(position, feature),
            };
            (
                OriginalTextScriptOr::Original(original_text_script(
                    script.original_file_name().to_string(),
                    script.original_text().to_string(),
                )),
                mapped,
                fidelity,
            )
        }
    }
}

pub struct DiagnosticOptions {
    pub report_style_checks_as_warnings: bool,
    pub related_information: bool,
    pub tag_value_set: Vec<i32>,
    pub visual_studio: bool,
}

pub fn diagnostic_to_lsp_pull(
    converters: &M5uConverters,
    diagnostic: &Diagnostic,
    report_style_checks_as_warnings: bool,
    client_diagnostic_caps: &LsprotoDiagnosticClientCapabilities,
    visual_studio: bool,
) -> LsprotoDiagnostic { ::tsox_core::fntrace::enter("diagnostic_to_lsp_pull"); 
    diagnostic_to_lsp(
        converters,
        diagnostic,
        DiagnosticOptions {
            report_style_checks_as_warnings,
            related_information: client_diagnostic_caps.related_information,
            tag_value_set: client_diagnostic_caps.tag_support_value_set.clone(),
            visual_studio,
        },
    )
}

pub fn diagnostic_to_lsp_push(
    converters: &M5uConverters,
    diagnostic: &Diagnostic,
    client_diagnostic_caps: &LsprotoDiagnosticClientCapabilities,
    visual_studio: bool,
) -> LsprotoDiagnostic { ::tsox_core::fntrace::enter("diagnostic_to_lsp_push"); 
    diagnostic_to_lsp(
        converters,
        diagnostic,
        DiagnosticOptions {
            report_style_checks_as_warnings: false,
            related_information: client_diagnostic_caps.related_information,
            tag_value_set: client_diagnostic_caps.tag_support_value_set.clone(),
            visual_studio,
        },
    )
}

pub struct LsprotoDiagnosticClientCapabilities {
    pub related_information: bool,
    pub tag_support_value_set: Vec<i32>,
}

pub const DIAGNOSTIC_TAG_UNNECESSARY: i32 = 1;
pub const DIAGNOSTIC_TAG_DEPRECATED: i32 = 2;

pub const DIAGNOSTIC_SEVERITY_ERROR: i32 = 1;
pub const DIAGNOSTIC_SEVERITY_WARNING: i32 = 2;
pub const DIAGNOSTIC_SEVERITY_INFORMATION: i32 = 3;
pub const DIAGNOSTIC_SEVERITY_HINT: i32 = 4;

pub struct LsprotoDiagnostic {
    pub range: M5uRange,
    pub code: Option<crate::lsp::lsproto_lsp::IntegerOrString>,
    pub severity: Option<i32>,
    pub message: String,
    pub source: Option<String>,
    pub related_information: Option<Vec<LsprotoDiagnosticRelatedInformation>>,
    pub tags: Option<Vec<i32>>,
}

pub struct LsprotoDiagnosticRelatedInformation {
    pub location: Location,
    pub message: String,
}

pub fn diagnostic_to_lsp(
    converters: &M5uConverters,
    diagnostic: &Diagnostic,
    opts: DiagnosticOptions,
) -> LsprotoDiagnostic { ::tsox_core::fntrace::enter("diagnostic_to_lsp"); 
    let mut severity = diagnostic_severity(diagnostic.category());

    if opts.report_style_checks_as_warnings
        && severity == DIAGNOSTIC_SEVERITY_ERROR
        && style_check_diagnostics_has(diagnostic.code())
    {
        severity = DIAGNOSTIC_SEVERITY_WARNING;
    }

    let mut related_information: Vec<LsprotoDiagnosticRelatedInformation> = Vec::new();
    if opts.related_information {
        for related in diagnostic.related_information() {
            let (script, loc) =
                diagnostic_script_and_range(related.file.as_ref(), related.loc, related.source());
            let (mut related_range, fidelity) = converters.to_lsp_range(&script, loc);
            if fidelity == SPANMAP_FIDELITY_NONE {
                related_range = M5uRange::default();
            }
            related_information.push(LsprotoDiagnosticRelatedInformation {
                location: Location {
                    uri: DocumentUri(file_name_to_document_uri(
                        related
                            .file
                            .as_ref()
                            .map_or("", |f| source_file_original_file_name(f)),
                    )),
                    range: related_range,
                },
                message: related_message(related),
            });
        }
    }

    let mut tags: Vec<i32> = Vec::new();
    if !opts.tag_value_set.is_empty()
        && (diagnostic.reports_unnecessary() || diagnostic.reports_deprecated())
    {
        if diagnostic.reports_unnecessary() && opts.tag_value_set.contains(&DIAGNOSTIC_TAG_UNNECESSARY)
        {
            tags.push(DIAGNOSTIC_TAG_UNNECESSARY);
        }
        if diagnostic.reports_deprecated() && opts.tag_value_set.contains(&DIAGNOSTIC_TAG_DEPRECATED)
        {
            tags.push(DIAGNOSTIC_TAG_DEPRECATED);
        }
    }

    let mut lsp_range = M5uRange::default();
    if let Some(file) = diagnostic.file() {
        let (script, loc) = diagnostic_script_and_range(Some(&file), diagnostic.loc(), diagnostic.source());
        let (range, fidelity) = converters.to_lsp_range(&script, loc);
        if fidelity != SPANMAP_FIDELITY_NONE {
            lsp_range = range;
        }
    }

    let source_text = if diagnostic.source().is_empty() {
        "ts".to_string()
    } else {
        diagnostic.source().to_string()
    };
    let code = if opts.visual_studio {
        crate::lsp::lsproto_lsp::IntegerOrString {
            integer: None,
            string: Some(format!("TS{}", diagnostic.code())),
        }
    } else {
        crate::lsp::lsproto_lsp::IntegerOrString {
            integer: Some(diagnostic.code() as i64),
            string: None,
        }
    };

    LsprotoDiagnostic {
        range: lsp_range,
        code: Some(code),
        severity: Some(severity),
        message: message_chain_to_string(diagnostic),
        source: Some(source_text),
        related_information: ptr_to_slice_if_non_empty(related_information),
        tags: ptr_to_slice_if_non_empty(tags),
    }
}

pub fn diagnostic_script_and_range<'a>(
    file: Option<&'a Arc<SourceFile>>,
    loc: tsox_core::core::text::TextRange,
    source: &str,
) -> (OriginalTextScriptOwn, tsox_core::core::text::TextRange) { ::tsox_core::fntrace::enter("diagnostic_script_and_range"); 
    let Some(file) = file else {
        return (OriginalTextScriptOwn::SourceFile, loc);
    };
    if source_file_span_map(file).is_none() {
        return (OriginalTextScriptOwn::SourceFile, loc);
    }
    let original = original_text_script(
        source_file_original_file_name(file).to_string(),
        source_file_original_text(file).to_string(),
    );
    if !source.is_empty() {
        return (OriginalTextScriptOwn::Original(original), loc);
    }
    let (mapped, fidelity) = source_file_span_map(file)
        .unwrap()
        .virtual_to_original_span(loc);
    if fidelity == SPANMAP_FIDELITY_NONE {
        return (
            OriginalTextScriptOwn::Original(original),
            tsox_core::core::text::TextRange::new(0, 0),
        );
    }
    (OriginalTextScriptOwn::Original(original), mapped)
}

pub enum OriginalTextScriptOwn {
    SourceFile,
    Original(OriginalTextScript),
}

impl M5uScript for OriginalTextScriptOwn {
    fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        match self {
            OriginalTextScriptOwn::SourceFile => "",
            OriginalTextScriptOwn::Original(s) => s.file_name(),
        }
    }

    fn original_file_name(&self) -> &str { ::tsox_core::fntrace::enter("original_file_name"); 
        match self {
            OriginalTextScriptOwn::SourceFile => "",
            OriginalTextScriptOwn::Original(s) => s.original_file_name(),
        }
    }

    fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        match self {
            OriginalTextScriptOwn::SourceFile => "",
            OriginalTextScriptOwn::Original(s) => s.text(),
        }
    }

    fn span_map(&self) -> Option<&dyn M5uSpanMap> { ::tsox_core::fntrace::enter("span_map"); 
        match self {
            OriginalTextScriptOwn::SourceFile => None,
            OriginalTextScriptOwn::Original(s) => s.span_map(),
        }
    }

    fn original_text(&self) -> &str { ::tsox_core::fntrace::enter("original_text"); 
        match self {
            OriginalTextScriptOwn::SourceFile => "",
            OriginalTextScriptOwn::Original(s) => s.original_text(),
        }
    }
}

pub fn diagnostic_severity(category: tsox_core::diagnostics::Category) -> i32 { ::tsox_core::fntrace::enter("diagnostic_severity"); 
    match category {
        tsox_core::diagnostics::Category::Warning => DIAGNOSTIC_SEVERITY_WARNING,
        tsox_core::diagnostics::Category::Error => DIAGNOSTIC_SEVERITY_ERROR,
        tsox_core::diagnostics::Category::Suggestion => DIAGNOSTIC_SEVERITY_HINT,
        tsox_core::diagnostics::Category::Message => DIAGNOSTIC_SEVERITY_INFORMATION,
    }
}

pub fn message_chain_to_string(diagnostic: &Diagnostic) -> String { ::tsox_core::fntrace::enter("message_chain_to_string"); 
    if diagnostic.message_chain().is_empty() {
        return diagnostic.display_string();
    }
    diagnostic_writer_write_flattened_ast_diagnostic_message(diagnostic)
}

pub fn ptr_to_slice_if_non_empty<T>(s: Vec<T>) -> Option<Vec<T>> { ::tsox_core::fntrace::enter("ptr_to_slice_if_non_empty"); 
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

pub fn style_check_diagnostics_has(_code: i32) -> bool { ::tsox_core::fntrace::enter("style_check_diagnostics_has"); 
    false
}

pub fn related_message(related: &Diagnostic) -> String { ::tsox_core::fntrace::enter("related_message"); 
    related.display_string()
}

pub fn diagnostic_writer_write_flattened_ast_diagnostic_message(diagnostic: &Diagnostic) -> String { ::tsox_core::fntrace::enter("diagnostic_writer_write_flattened_ast_diagnostic_message"); 
    use std::io::Write;
    let mut buf: Vec<u8> = Vec::new();
    tsox_frontend::diagnosticwriter::mig::x12::write_flattened_ast_diagnostic_message(
        &mut buf,
        diagnostic,
        "\n",
        &tsox_core::locale::Locale::default_locale(),
    );
    String::from_utf8(buf).unwrap_or_default()
}

pub fn source_file_span_map(_file: &Arc<SourceFile>) -> Option<&'static dyn M5uSpanMap> { ::tsox_core::fntrace::enter("source_file_span_map"); 
    None
}

pub fn source_file_original_file_name(file: &Arc<SourceFile>) -> &str { ::tsox_core::fntrace::enter("source_file_original_file_name"); 
    &file.file_name
}

pub fn source_file_original_text(file: &Arc<SourceFile>) -> &str { ::tsox_core::fntrace::enter("source_file_original_text"); 
    &file.text
}

pub fn source_file_supplemental_source_files(_file: &Arc<SourceFile>) -> Vec<Arc<SourceFile>> { ::tsox_core::fntrace::enter("source_file_supplemental_source_files"); 
    Vec::new()
}

pub fn convention_line_and_character_to_position(
    c: &M5uConverters,
    script: &dyn M5uScript,
    line_and_character: &Position,
) -> usize { ::tsox_core::fntrace::enter("convention_line_and_character_to_position"); 
    let existing = crate::ls::lsconv_converters::Converters::new(c.position_encoding);
    let view = PlainScriptView {
        file_name: script.file_name().to_string(),
        text: script.text().to_string(),
    };
    existing.line_and_character_to_position(&view, line_and_character)
}

pub fn convention_position_to_line_and_character(
    c: &M5uConverters,
    script: &dyn M5uScript,
    position: usize,
) -> Position { ::tsox_core::fntrace::enter("convention_position_to_line_and_character"); 
    let existing = crate::ls::lsconv_converters::Converters::new(c.position_encoding);
    let view = PlainScriptView {
        file_name: script.file_name().to_string(),
        text: script.text().to_string(),
    };
    existing.position_to_line_and_character(&view, position)
}

pub struct PlainScriptView {
    pub file_name: String,
    pub text: String,
}

impl crate::ls::lsconv_converters::Script for PlainScriptView {
    fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        &self.file_name
    }

    fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        &self.text
    }
}
