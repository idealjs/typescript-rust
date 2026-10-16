#![allow(dead_code, unused_imports, unused_variables)]

use tsox_core::core::compiler_options_kinds::ModuleResolutionKind;

pub fn module_resolution_supports_package_json_exports_and_imports(
    module_resolution: ModuleResolutionKind,
) -> bool { ::tsox_core::fntrace::enter("module_resolution_supports_package_json_exports_and_imports"); 
    (module_resolution as i32 >= ModuleResolutionKind::Node16 as i32
        && module_resolution as i32 <= ModuleResolutionKind::NodeNext as i32)
        || module_resolution as i32 == ModuleResolutionKind::Bundler as i32
}
