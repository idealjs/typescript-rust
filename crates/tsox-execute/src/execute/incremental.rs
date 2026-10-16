#![allow(unused_imports, dead_code)]

pub use crate::mig::m4z2::Program;

use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::tristate::Tristate;

/// Go: execute/incremental.FileEmitKind(u32 位掩码)。组合值以显式枚举承载,
/// 仅覆盖 GetFileEmitKind 可构造出的取值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FileEmitKind {
    None = 0,
    Js = 1 << 0,
    JsMap = 1 << 1,
    JsInlineMap = 1 << 2,
    DtsErrors = 1 << 3,
    DtsEmit = 1 << 4,
    DtsMap = 1 << 5,
    Dts = (1 << 3) | (1 << 4),
    AllJs = (1 << 0) | (1 << 1) | (1 << 2),
    AllDtsEmit = (1 << 4) | (1 << 5),
    AllDts = (1 << 3) | (1 << 4) | (1 << 5),
    All = (1 << 0)
        | (1 << 1)
        | (1 << 2)
        | (1 << 3)
        | (1 << 4)
        | (1 << 5),
    JsJsMap = (1 << 0) | (1 << 1),
    JsJsInlineMap = (1 << 0) | (1 << 2),
    JsDts = (1 << 0) | (1 << 3) | (1 << 4),
    JsJsMapDts = (1 << 0) | (1 << 1) | (1 << 3) | (1 << 4),
    JsJsInlineMapDts = (1 << 0) | (1 << 2) | (1 << 3) | (1 << 4),
    AllJsDts = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4),
    JsDtsMap = (1 << 0) | (1 << 5),
    JsJsMapDtsMap = (1 << 0) | (1 << 1) | (1 << 5),
    JsJsInlineMapDtsMap = (1 << 0) | (1 << 2) | (1 << 5),
    AllJsDtsMap = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 5),
    JsAllDts = (1 << 0) | (1 << 3) | (1 << 4) | (1 << 5),
    JsJsMapAllDts = (1 << 0) | (1 << 1) | (1 << 3) | (1 << 4) | (1 << 5),
    JsJsInlineMapAllDts = (1 << 0) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5),
}

impl FileEmitKind {
    fn bits(self) -> u32 { ::tsox_core::fntrace::enter("bits"); 
        self as u32
    }

    fn from_bits(bits: u32) -> FileEmitKind { ::tsox_core::fntrace::enter("from_bits"); 
        match bits {
            b if b == FileEmitKind::None as u32 => FileEmitKind::None,
            b if b == FileEmitKind::Js as u32 => FileEmitKind::Js,
            b if b == FileEmitKind::JsMap as u32 => FileEmitKind::JsMap,
            b if b == FileEmitKind::JsInlineMap as u32 => FileEmitKind::JsInlineMap,
            b if b == FileEmitKind::DtsErrors as u32 => FileEmitKind::DtsErrors,
            b if b == FileEmitKind::DtsEmit as u32 => FileEmitKind::DtsEmit,
            b if b == FileEmitKind::DtsMap as u32 => FileEmitKind::DtsMap,
            b if b == FileEmitKind::Dts as u32 => FileEmitKind::Dts,
            b if b == FileEmitKind::AllJs as u32 => FileEmitKind::AllJs,
            b if b == FileEmitKind::AllDtsEmit as u32 => FileEmitKind::AllDtsEmit,
            b if b == FileEmitKind::AllDts as u32 => FileEmitKind::AllDts,
            b if b == FileEmitKind::All as u32 => FileEmitKind::All,
            b if b == FileEmitKind::JsJsMap as u32 => FileEmitKind::JsJsMap,
            b if b == FileEmitKind::JsJsInlineMap as u32 => FileEmitKind::JsJsInlineMap,
            b if b == FileEmitKind::JsDts as u32 => FileEmitKind::JsDts,
            b if b == FileEmitKind::JsJsMapDts as u32 => FileEmitKind::JsJsMapDts,
            b if b == FileEmitKind::JsJsInlineMapDts as u32 => FileEmitKind::JsJsInlineMapDts,
            b if b == FileEmitKind::AllJsDts as u32 => FileEmitKind::AllJsDts,
            b if b == FileEmitKind::JsDtsMap as u32 => FileEmitKind::JsDtsMap,
            b if b == FileEmitKind::JsJsMapDtsMap as u32 => FileEmitKind::JsJsMapDtsMap,
            b if b == FileEmitKind::JsJsInlineMapDtsMap as u32 => FileEmitKind::JsJsInlineMapDtsMap,
            b if b == FileEmitKind::AllJsDtsMap as u32 => FileEmitKind::AllJsDtsMap,
            b if b == FileEmitKind::JsAllDts as u32 => FileEmitKind::JsAllDts,
            b if b == FileEmitKind::JsJsMapAllDts as u32 => FileEmitKind::JsJsMapAllDts,
            b if b == FileEmitKind::JsJsInlineMapAllDts as u32 => FileEmitKind::JsJsInlineMapAllDts,
            _ => FileEmitKind::None,
        }
    }

    fn or(self, other: FileEmitKind) -> FileEmitKind { ::tsox_core::fntrace::enter("or"); 
        FileEmitKind::from_bits(self.bits() | other.bits())
    }

    fn and(self, other: FileEmitKind) -> FileEmitKind { ::tsox_core::fntrace::enter("and"); 
        FileEmitKind::from_bits(self.bits() & other.bits())
    }
}

/// Go: incremental.GetFileEmitKind
pub fn get_file_emit_kind(options: &CompilerOptions) -> FileEmitKind { ::tsox_core::fntrace::enter("get_file_emit_kind"); 
    let mut result = FileEmitKind::Js;
    if options.source_map.is_true() {
        result = result.or(FileEmitKind::JsMap);
    }
    if options.inline_source_map.is_true() {
        result = result.or(FileEmitKind::JsInlineMap);
    }
    if options.get_emit_declarations() {
        result = result.or(FileEmitKind::Dts);
    }
    if options.declaration_map.is_true() {
        result = result.or(FileEmitKind::DtsMap);
    }
    if options.emit_declaration_only.is_true() {
        result = result.and(FileEmitKind::AllDts);
    }
    result
}
