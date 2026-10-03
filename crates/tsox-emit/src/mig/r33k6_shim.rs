#![allow(dead_code)]

use tsox_core::core::compiler_options_kinds::JsxEmit;
use tsox_core::tspath;
use tsox_core::tspath::Path as TsPath;

pub trait HasFileName {
    fn file_name(&self) -> &str;
    fn path(&self) -> &TsPath;
}

impl HasFileName for tsox_frontend::ast::mig::m3g_3::HasFileNameImpl {
    fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        self.file_name()
    }

    fn path(&self) -> &TsPath { ::tsox_core::fntrace::enter("path"); 
        self.path()
    }
}

pub struct EmitHost;

pub struct ResolveModuleNameResolutionHost;

pub struct Visitor;

pub fn get_output_extension(file_name: &str, jsx: JsxEmit) -> String { ::tsox_core::fntrace::enter("get_output_extension"); 
    if tspath::file_extension_is(file_name, tspath::EXTENSION_JSON) {
        return tspath::EXTENSION_JSON.to_string();
    }
    if jsx == JsxEmit::Preserve
        && tspath::file_extension_is_one_of(file_name, &[tspath::EXTENSION_JSX, tspath::EXTENSION_TSX])
    {
        return tspath::EXTENSION_JSX.to_string();
    }
    if tspath::file_extension_is_one_of(file_name, &[tspath::EXTENSION_MTS, tspath::EXTENSION_MJS]) {
        return tspath::EXTENSION_MJS.to_string();
    }
    if tspath::file_extension_is_one_of(file_name, &[tspath::EXTENSION_CTS, tspath::EXTENSION_CJS]) {
        return tspath::EXTENSION_CJS.to_string();
    }
    tspath::EXTENSION_JS.to_string()
}
