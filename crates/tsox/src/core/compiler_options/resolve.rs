use super::kinds::*;
use super::options::CompilerOptions;
use crate::core::tristate::Tristate;
use crate::tspath;

impl CompilerOptions {
    pub fn get_emit_script_target(&self) -> ScriptTarget {
        if self.target != ScriptTarget::None {
            self.target
        } else {
            ScriptTarget::LATEST_STANDARD
        }
    }

    pub fn get_emit_module_kind(&self) -> ModuleKind {
        if self.module != ModuleKind::None {
            return self.module;
        }
        let target = self.get_emit_script_target();
        if target == ScriptTarget::ESNext {
            ModuleKind::ESNext
        } else if target >= ScriptTarget::ES2022 {
            ModuleKind::ES2022
        } else if target >= ScriptTarget::ES2020 {
            ModuleKind::ES2020
        } else if target >= ScriptTarget::ES2015 {
            ModuleKind::ES2015
        } else {
            ModuleKind::CommonJS
        }
    }

    pub fn get_module_resolution_kind(&self) -> ModuleResolutionKind {
        match self.module_resolution {
            ModuleResolutionKind::Unknown | ModuleResolutionKind::Classic => {
                match self.get_emit_module_kind() {
                    ModuleKind::Node16 | ModuleKind::Node18 | ModuleKind::Node20 => {
                        ModuleResolutionKind::Node16
                    }
                    ModuleKind::NodeNext => ModuleResolutionKind::NodeNext,
                    _ => ModuleResolutionKind::Bundler,
                }
            }
            other => other,
        }
    }

    pub fn get_emit_module_detection_kind(&self) -> ModuleDetectionKind {
        if self.module_detection != ModuleDetectionKind::None {
            return self.module_detection;
        }
        let module_kind = self.get_emit_module_kind();
        if module_kind >= ModuleKind::Node16 && module_kind <= ModuleKind::NodeNext {
            ModuleDetectionKind::Force
        } else {
            ModuleDetectionKind::Auto
        }
    }

    pub fn get_resolve_package_json_exports(&self) -> bool {
        self.resolve_package_json_exports.is_true_or_unknown()
    }

    pub fn get_resolve_package_json_imports(&self) -> bool {
        self.resolve_package_json_imports.is_true_or_unknown()
    }

    pub fn get_allow_importing_ts_extensions(&self) -> bool {
        self.allow_importing_ts_extensions.is_true()
            || self.rewrite_relative_import_extensions.is_true()
    }

    pub fn allow_importing_ts_extensions_from(&self, file_name: &str) -> bool {
        self.get_allow_importing_ts_extensions() || tspath::is_declaration_file_name(file_name)
    }

    pub fn get_resolve_json_module(&self) -> bool {
        if self.resolve_json_module != Tristate::Unknown {
            return self.resolve_json_module == Tristate::True;
        }
        match self.get_emit_module_kind() {
            ModuleKind::Node20 | ModuleKind::NodeNext => true,
            _ => self.get_module_resolution_kind() == ModuleResolutionKind::Bundler,
        }
    }

    pub fn should_preserve_const_enums(&self) -> bool {
        self.preserve_const_enums == Tristate::True || self.get_isolated_modules()
    }

    pub fn get_allow_js(&self) -> bool {
        if self.allow_js != Tristate::Unknown {
            self.allow_js == Tristate::True
        } else {
            self.check_js == Tristate::True
        }
    }

    pub fn get_jsx_transform_enabled(&self) -> bool {
        matches!(
            self.jsx,
            JsxEmit::React | JsxEmit::ReactJSX | JsxEmit::ReactJSXDev
        )
    }

    pub fn get_strict_option_value(&self, value: Tristate) -> bool {
        if value != Tristate::Unknown {
            return value == Tristate::True;
        }

        self.strict != Tristate::False
    }

    pub fn get_isolated_modules(&self) -> bool {
        self.isolated_modules == Tristate::True || self.verbatim_module_syntax == Tristate::True
    }

    pub fn is_incremental(&self) -> bool {
        self.incremental.is_true() || self.composite.is_true()
    }

    pub fn get_emit_standard_class_fields(&self) -> bool {
        self.use_define_for_class_fields != Tristate::False
            && self.get_emit_script_target() >= ScriptTarget::ES2022
    }

    pub fn get_use_define_for_class_fields(&self) -> bool {
        if self.use_define_for_class_fields == Tristate::Unknown {
            self.get_emit_script_target() >= ScriptTarget::ES2022
        } else {
            self.use_define_for_class_fields == Tristate::True
        }
    }

    pub fn get_emit_declarations(&self) -> bool {
        self.declaration.is_true() || self.composite.is_true()
    }

    pub fn get_are_declaration_maps_enabled(&self) -> bool {
        self.declaration_map == Tristate::True && self.get_emit_declarations()
    }

    pub fn has_json_module_emit_enabled(&self) -> bool {
        !matches!(
            self.get_emit_module_kind(),
            ModuleKind::System | ModuleKind::UMD
        )
    }

    pub fn uses_wildcard_types(&self) -> bool {
        self.types.iter().any(|t| t == "*")
    }
}
