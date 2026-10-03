use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::tspath;

use crate::modulespecifiers::types::{ModuleSpecifierGenerationHost, ModuleSpecifierEnding};

use super::m3m::{
    get_js_extension_for_declaration_file_extension, get_js_extension_for_file,
    try_get_any_file_from_path, try_get_real_file_name_for_non_js_declaration_file_name,
};

pub struct SpecPair {
    pub ending: ModuleSpecifierEnding,
    pub value: String,
}

pub fn validate_ending(
    c: &SpecPair,
    relative_to_base_url: &str,
    compiler_options: &CompilerOptions,
    host: &dyn ModuleSpecifierGenerationHost,
) -> bool { ::tsox_core::fntrace::enter("validate_ending"); 
    c.ending != ModuleSpecifierEnding::Minimal
        || c.value
            == process_ending(
                relative_to_base_url,
                &[c.ending],
                compiler_options,
                host,
            )
}

pub fn process_ending(
    file_name: &str,
    allowed_endings: &[ModuleSpecifierEnding],
    options: &CompilerOptions,
    host: &dyn ModuleSpecifierGenerationHost,
) -> String { ::tsox_core::fntrace::enter("process_ending"); 
    if tspath::file_extension_is_one_of(
        file_name,
        &[
            tspath::EXTENSION_JSON,
            tspath::EXTENSION_MJS,
            tspath::EXTENSION_CJS,
        ],
    ) {
        return file_name.to_string();
    }

    let no_extension = tspath::remove_file_extension(file_name);
    if file_name == no_extension {
        return file_name.to_string();
    }

    let js_priority = allowed_endings
        .iter()
        .position(|e| *e == ModuleSpecifierEnding::JsExtension);
    let ts_priority = allowed_endings
        .iter()
        .position(|e| *e == ModuleSpecifierEnding::TsExtension);
    if tspath::file_extension_is_one_of(file_name, &[tspath::EXTENSION_MTS, tspath::EXTENSION_CTS])
    {
        if let (Some(tp), Some(jp)) = (ts_priority, js_priority) {
            if tp < jp {
                return file_name.to_string();
            }
        }
    }
    if tspath::file_extension_is_one_of(
        file_name,
        &[tspath::EXTENSION_DMTS, tspath::EXTENSION_DCTS],
    ) {
        let input_ext = tspath::get_declaration_file_extension(file_name);
        let ext = get_js_extension_for_declaration_file_extension(&input_ext);
        return format!("{}{}", tspath::remove_extension(file_name, &input_ext), ext);
    }
    if tspath::file_extension_is_one_of(file_name, &[tspath::EXTENSION_MTS, tspath::EXTENSION_CTS])
    {
        return format!("{}{}", no_extension, get_js_extension_for_file(file_name, options));
    }
    if !tspath::file_extension_is_one_of(file_name, &[tspath::EXTENSION_DTS])
        && tspath::file_extension_is_one_of(file_name, &[tspath::EXTENSION_TS])
        && file_name.contains(".d.")
    {
        // `foo.d.json.ts` and the like - remap back to `foo.json`
        let result = try_get_real_file_name_for_non_js_declaration_file_name(file_name);
        if !result.is_empty() {
            return result;
        }
    }

    match allowed_endings[0] {
        ModuleSpecifierEnding::Minimal => {
            let without_index = no_extension.strip_suffix("/index").unwrap_or(&no_extension);
            if without_index != no_extension && try_get_any_file_from_path(host, without_index) {
                // Can't remove index if there's a file by the same name as the directory.
                return no_extension;
            }
            without_index.to_string()
        }
        ModuleSpecifierEnding::Index => no_extension,
        ModuleSpecifierEnding::JsExtension => {
            format!("{}{}", no_extension, get_js_extension_for_file(file_name, options))
        }
        ModuleSpecifierEnding::TsExtension => {
            // For now, we don't know if this import is going to be type-only, which means we don't
            // know if a .d.ts extension is valid, so use no extension or a .js extension
            if tspath::is_declaration_file_name(file_name) {
                let mut extensionless_priority = None;
                for (i, e) in allowed_endings.iter().enumerate() {
                    if matches!(
                        e,
                        ModuleSpecifierEnding::Minimal | ModuleSpecifierEnding::Index
                    ) {
                        extensionless_priority = Some(i);
                        break;
                    }
                }
                if let (Some(ep), Some(jp)) = (extensionless_priority, js_priority) {
                    if ep < jp {
                        return no_extension;
                    }
                }
                return format!(
                    "{}{}",
                    no_extension,
                    get_js_extension_for_file(file_name, options)
                );
            }
            file_name.to_string()
        }
    }
}
