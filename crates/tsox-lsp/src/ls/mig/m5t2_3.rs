#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::language_service::LanguageService;
use crate::ls::lsutil_symbol_display::{ScriptElementKind, ScriptElementKindModifier};
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{Node, SourceFile, Symbol};

pub use super::m5s::{SpanFeature, SpanFidelity, VSClassifiedTextRun};
use crate::lsp::lsproto_lsp_basic::Range;
use crate::mig::m5u_conv::{new_converters as new_m5u_converters, SourceFileScriptView};

fn m5t23_m5u_converters(
    encoding: crate::ls::lsconv_converters::PositionEncodingKind,
) -> crate::mig::m5u_conv::M5uConverters { ::tsox_core::fntrace::enter("m5t23_m5u_converters"); 
    new_m5u_converters(
        encoding,
        Box::new(crate::ls::lsconv_linemap::compute_lsp_line_starts),
    )
}

/// Go projection.SpanMap()：span map 挂在 SourceFile 上，Rust ast 层尚未
/// 落地该字段（m6b_2::SpanMap 亦无生产方），先返 None，见 progress_notes_r59A.md 交接
fn source_file_span_map_m5t23(_file: &Arc<SourceFile>) -> Option<crate::mig::m6b_2::SpanMap> { ::tsox_core::fntrace::enter("source_file_span_map_m5t23"); 
    None
}

pub const CLASSIFICATION_TYPE_NAME_TEXT: &str = "text";

pub const VS_IMAGE_CATALOG_GUID: &str = "ae27a6b0-e345-4288-96df-5eaf394ee369";

pub const IMAGE_ID_WARNING: i32 = 0x00000637;
pub const IMAGE_ID_KEYWORD: i32 = 0x00000635;
pub const IMAGE_ID_MODULE_PRIVATE: i32 = 0x0000077D;
pub const IMAGE_ID_MODULE_PROTECTED: i32 = 0x0000077E;
pub const IMAGE_ID_MODULE_PUBLIC: i32 = 0x0000077F;
pub const IMAGE_ID_TYPE: i32 = 0x00000CA1;
pub const IMAGE_ID_NAMESPACE: i32 = 0x0000079F;
pub const IMAGE_ID_CLASS_PRIVATE: i32 = 0x000001D7;
pub const IMAGE_ID_CLASS_PROTECTED: i32 = 0x000001D8;
pub const IMAGE_ID_CLASS_PUBLIC: i32 = 0x000001D9;
pub const IMAGE_ID_INTERFACE_PRIVATE: i32 = 0x00000646;
pub const IMAGE_ID_INTERFACE_PROTECTED: i32 = 0x00000647;
pub const IMAGE_ID_INTERFACE_PUBLIC: i32 = 0x00000648;
pub const IMAGE_ID_ENUM_PRIVATE: i32 = 0x00000469;
pub const IMAGE_ID_ENUM_PROTECTED: i32 = 0x0000046A;
pub const IMAGE_ID_ENUM_PUBLIC: i32 = 0x0000046B;
pub const IMAGE_ID_ENUM_MEMBER: i32 = 0x00000465;
pub const IMAGE_ID_LOCAL_VARIABLE: i32 = 0x000006D3;
pub const IMAGE_ID_PROPERTY_PRIVATE: i32 = 0x00000982;
pub const IMAGE_ID_PROPERTY_PROTECTED: i32 = 0x00000983;
pub const IMAGE_ID_PROPERTY_PUBLIC: i32 = 0x00000984;
pub const IMAGE_ID_METHOD_PRIVATE: i32 = 0x00000756;
pub const IMAGE_ID_METHOD_PROTECTED: i32 = 0x00000757;
pub const IMAGE_ID_METHOD_PUBLIC: i32 = 0x00000758;
pub const IMAGE_ID_LABEL: i32 = 0x0000067D;
pub const IMAGE_ID_ASSEMBLY: i32 = 0x000000C4;
pub const IMAGE_ID_CONSTANT_PRIVATE: i32 = 0x0000026A;
pub const IMAGE_ID_CONSTANT_PROTECTED: i32 = 0x0000026B;
pub const IMAGE_ID_CONSTANT_PUBLIC: i32 = 0x0000026C;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VSImageId {
    pub guid: String,
    pub id: i32,
}

#[derive(Debug, Clone)]
pub struct VSImageElement {
    pub image_id: VSImageId,
}

#[derive(Debug, Clone, Default)]
pub struct VSClassifiedTextElement {
    pub runs: Vec<VSClassifiedTextRun>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSContainerElementStyle {
    Wrapped,
    Stacked,
}

#[derive(Debug, Clone)]
pub enum VSImageElementOrClassifiedTextElementOrContainerElement {
    ImageElement(VSImageElement),
    ClassifiedTextElement(VSClassifiedTextElement),
    ContainerElement(VSContainerElement),
}

#[derive(Debug, Clone)]
pub struct VSContainerElement {
    pub style: VSContainerElementStyle,
    pub elements: Vec<VSImageElementOrClassifiedTextElementOrContainerElement>,
}

pub fn new_vs_image_id(id: i32) -> VSImageId { ::tsox_core::fntrace::enter("new_vs_image_id"); 
    VSImageId {
        guid: VS_IMAGE_CATALOG_GUID.to_string(),
        id,
    }
}

pub fn get_vs_hover_image_id(kind: ScriptElementKind, modifiers: ScriptElementKindModifier) -> Option<VSImageId> { ::tsox_core::fntrace::enter("get_vs_hover_image_id"); 
    let is_private = modifiers & ScriptElementKindModifier::PRIVATE.0 != 0;
    let is_protected = modifiers & ScriptElementKindModifier::PROTECTED.0 != 0;
    let pick = |private: i32, protected: i32, public: i32| {
        if is_private {
            new_vs_image_id(private)
        } else if is_protected {
            new_vs_image_id(protected)
        } else {
            new_vs_image_id(public)
        }
    };
    Some(match kind {
        ScriptElementKind::Warning => new_vs_image_id(IMAGE_ID_WARNING),
        ScriptElementKind::Keyword => new_vs_image_id(IMAGE_ID_KEYWORD),
        ScriptElementKind::ScriptElement => pick(IMAGE_ID_MODULE_PRIVATE, IMAGE_ID_MODULE_PROTECTED, IMAGE_ID_MODULE_PUBLIC),
        ScriptElementKind::PrimitiveType => new_vs_image_id(IMAGE_ID_TYPE),
        ScriptElementKind::ModuleElement => new_vs_image_id(IMAGE_ID_NAMESPACE),
        ScriptElementKind::ConstructorImplementationElement
        | ScriptElementKind::ClassElement
        | ScriptElementKind::LocalClassElement
        | ScriptElementKind::TypeElement => pick(IMAGE_ID_CLASS_PRIVATE, IMAGE_ID_CLASS_PROTECTED, IMAGE_ID_CLASS_PUBLIC),
        ScriptElementKind::InterfaceElement => {
            pick(IMAGE_ID_INTERFACE_PRIVATE, IMAGE_ID_INTERFACE_PROTECTED, IMAGE_ID_INTERFACE_PUBLIC)
        }
        ScriptElementKind::EnumElement => pick(IMAGE_ID_ENUM_PRIVATE, IMAGE_ID_ENUM_PROTECTED, IMAGE_ID_ENUM_PUBLIC),
        ScriptElementKind::EnumMemberElement => new_vs_image_id(IMAGE_ID_ENUM_MEMBER),
        ScriptElementKind::ParameterElement
        | ScriptElementKind::VariableElement
        | ScriptElementKind::LocalVariableElement
        | ScriptElementKind::VariableUsingElement
        | ScriptElementKind::VariableAwaitUsingElement
        | ScriptElementKind::LetElement
        | ScriptElementKind::String => new_vs_image_id(IMAGE_ID_LOCAL_VARIABLE),
        ScriptElementKind::ConstElement => pick(IMAGE_ID_CONSTANT_PRIVATE, IMAGE_ID_CONSTANT_PROTECTED, IMAGE_ID_CONSTANT_PUBLIC),
        ScriptElementKind::MemberGetAccessorElement
        | ScriptElementKind::MemberSetAccessorElement
        | ScriptElementKind::MemberVariableElement
        | ScriptElementKind::MemberAccessorVariableElement => {
            pick(IMAGE_ID_PROPERTY_PRIVATE, IMAGE_ID_PROPERTY_PROTECTED, IMAGE_ID_PROPERTY_PUBLIC)
        }
        ScriptElementKind::FunctionElement
        | ScriptElementKind::LocalFunctionElement
        | ScriptElementKind::MemberFunctionElement
        | ScriptElementKind::CallSignatureElement
        | ScriptElementKind::IndexSignatureElement
        | ScriptElementKind::ConstructSignatureElement => {
            pick(IMAGE_ID_METHOD_PRIVATE, IMAGE_ID_METHOD_PROTECTED, IMAGE_ID_METHOD_PUBLIC)
        }
        ScriptElementKind::TypeParameterElement => new_vs_image_id(IMAGE_ID_TYPE),
        ScriptElementKind::Label => new_vs_image_id(IMAGE_ID_LABEL),
        ScriptElementKind::Alias => new_vs_image_id(IMAGE_ID_MODULE_PUBLIC),
        _ => return None,
    })
}

pub fn build_vs_hover_raw_content(
    image_id: &VSImageId,
    quick_info_runs: &[VSClassifiedTextRun],
    documentation_runs: &[VSClassifiedTextRun],
) -> Option<VSContainerElement> { ::tsox_core::fntrace::enter("build_vs_hover_raw_content"); 
    if quick_info_runs.is_empty() {
        return None;
    }
    let display_line = VSContainerElement {
        style: VSContainerElementStyle::Wrapped,
        elements: vec![
            VSImageElementOrClassifiedTextElementOrContainerElement::ImageElement(VSImageElement {
                image_id: image_id.clone(),
            }),
            VSImageElementOrClassifiedTextElementOrContainerElement::ClassifiedTextElement(VSClassifiedTextElement {
                runs: quick_info_runs.to_vec(),
            }),
        ],
    };
    if documentation_runs.is_empty() {
        return Some(display_line);
    }
    Some(VSContainerElement {
        style: VSContainerElementStyle::Stacked,
        elements: vec![
            VSImageElementOrClassifiedTextElementOrContainerElement::ContainerElement(display_line),
            VSImageElementOrClassifiedTextElementOrContainerElement::ClassifiedTextElement(VSClassifiedTextElement {
                runs: documentation_runs.to_vec(),
            }),
        ],
    })
}

pub struct MappedFormattingRange {
    pub projection: Arc<SourceFile>,
    pub segment: crate::mig::m6b_2::Segment,
    pub original_range: TextRange,
}

impl LanguageService {
    pub fn get_formatting_edits_for_mapped_range(
        &self,
        file: &Arc<SourceFile>,
        options: &crate::ls::lsutil_format_code_options::FormatCodeSettings,
        original_range: TextRange,
    ) -> Vec<crate::lsp::lsproto_lsp_basic::TextEdit> { ::tsox_core::fntrace::enter("get_formatting_edits_for_mapped_range"); 
        let mut projections = vec![file.clone()];
        projections.extend(file.supplemental_source_files());
        let mut candidates: Vec<MappedFormattingRange> = Vec::new();
        for projection in &projections {
            let Some(span_map) = source_file_span_map_m5t23(projection) else {
                continue;
            };
            for segment in span_map.segments() {
                if segment.kind != crate::mig::m6b_2::Kind::Verbatim
                    || segment.features & crate::mig::m6b_2::FEATURE_FORMATTING == 0
                {
                    continue;
                }
                let original_start = original_range.pos().max(segment.original_start as usize);
                let original_end = original_range.end().min(segment.original_end as usize);
                if original_start >= original_end {
                    continue;
                }
                candidates.push(MappedFormattingRange {
                    projection: projection.clone(),
                    segment: segment.clone(),
                    original_range: TextRange::new(original_start, original_end),
                });
            }
        }

        let mut edits: Vec<crate::lsp::lsproto_lsp_basic::TextEdit> = Vec::new();
        for candidate in non_overlapping_formatting_ranges(candidates) {
            let virtual_range = TextRange::new(
                candidate.segment.virtual_start + candidate.original_range.pos()
                    - candidate.segment.original_start,
                candidate.segment.virtual_start + candidate.original_range.end()
                    - candidate.segment.original_start,
            );
            for change in self.get_formatting_edits_for_range(&candidate.projection, options, virtual_range) {
                if change.range.pos() < virtual_range.pos() || change.range.end() > virtual_range.end() {
                    continue;
                }
                // Go：l.converters.ToLSPRangeForFeature(projection, …, FeatureFormatting)；
                // M5uConverters 与 lsconv Converters 是两套（编码经宿主协商），
                // 这里按宿主实际编码 Utf16 构造，见 progress_notes_r59A.md 交接
                let script_view = SourceFileScriptView { file: Arc::clone(&candidate.projection) };
                let (lsp_range, fidelity) = m5t23_m5u_converters(
                    crate::ls::lsconv_converters::PositionEncodingKind::Utf16,
                )
                .to_lsp_range_for_feature(
                    &script_view,
                    TextRange::new(change.range.pos(), change.range.end()),
                    crate::mig::m6b_2::FEATURE_FORMATTING as crate::mig::m5u_conv::SpanMapFeature,
                );
                if fidelity != crate::mig::m5u_conv::SPANMAP_FIDELITY_EXACT {
                    continue;
                }
                edits.push(crate::lsp::lsproto_lsp_basic::TextEdit {
                    range: lsp_range,
                    new_text: change.new_text,
                });
            }
        }
        edits.sort_by(|a, b| {
            let c = crate::lsp::lsproto_util::compare_ranges(&a.range, &b.range);
            if c != std::cmp::Ordering::Equal {
                return c;
            }
            a.new_text.cmp(&b.new_text)
        });
        edits
    }
}

pub fn non_overlapping_formatting_ranges(candidates: Vec<MappedFormattingRange>) -> Vec<MappedFormattingRange> { ::tsox_core::fntrace::enter("non_overlapping_formatting_ranges"); 
    let mut candidates = candidates;
    candidates.sort_by(|a, b| {
        let c = a.original_range.pos().cmp(&b.original_range.pos());
        if c != std::cmp::Ordering::Equal {
            return c;
        }
        b.original_range.end().cmp(&a.original_range.end())
    });
    let mut result: Vec<MappedFormattingRange> = Vec::new();
    for mut candidate in candidates {
        if let Some(last) = result.last() {
            let new_pos = candidate.original_range.pos().max(last.original_range.end());
            candidate.original_range = TextRange::new(new_pos, candidate.original_range.end());
        }
        if !candidate.original_range.is_empty() {
            result.push(candidate);
        }
    }
    result
}
