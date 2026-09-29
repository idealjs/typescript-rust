#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::SourceFile;

use crate::ls::language_service::LanguageService;

// NewLanguageService: crate::ls::language_service::LanguageService::new 已是孪生实现,不重复移植。

impl LanguageService {
    pub fn get_prepared_auto_import_view(
        &self,
        from_file: &Arc<SourceFile>,
    ) -> Result<crate::ls::autoimport_view::View, NeedsAutoImports> {
        let registry = host_registry_placeholder(&self.host.auto_import_registry(), self.user_preferences());
        let mut registry_file = Arc::clone(from_file);
        if let Some(canonical) = canonical_source_file(from_file) {
            registry_file = canonical;
        }
        if !registry.is_prepared_for_importing_file(
            &registry_file.file_name,
            &self.project_path,
            self.user_preferences(),
        ) {
            return Err(NEEDS_AUTO_IMPORTS);
        }

        let view = crate::ls::autoimport_view::View::new(
            registry,
            Arc::clone(from_file),
            self.project_path.clone(),
            Arc::clone(&self.program),
            self.user_preferences().module_specifier_preferences(),
        );
        Ok(view)
    }

    pub fn get_current_auto_import_view(
        &self,
        from_file: &Arc<SourceFile>,
    ) -> crate::ls::autoimport_view::View {
        crate::ls::autoimport_view::View::new(
            host_registry_placeholder(
                &self.host.auto_import_registry(),
                self.user_preferences(),
            ),
            Arc::clone(from_file),
            self.project_path.clone(),
            Arc::clone(&self.program),
            self.user_preferences().module_specifier_preferences(),
        )
    }
}

// 宿主 Host::auto_import_registry 现为空孪生(ls/host.rs::AutoImportRegistry 单元结构,无注册表数据),
// 按空 Registry 语义落地: projects/node_modules 均无桶,is_prepared_for_importing_file 恒 false,
// 与 Go 侧未初始化注册表行为一致。宿主注册表落地后此处改直接透传 Arc<Registry>。
fn host_registry_placeholder(
    registry: &crate::ls::host::AutoImportRegistry,
    preferences: &crate::ls::lsutil_user_preferences::UserPreferences,
) -> Arc<crate::ls::autoimport_registry_registry_impl::Registry> {
    let _ = registry;
    Arc::new(crate::ls::autoimport_registry_registry_impl::Registry::new(
        Box::new(|name: &str| tsox_core::tspath::to_path(name, "", true)),
        preferences.clone(),
    ))
}

fn canonical_source_file(_file: &Arc<SourceFile>) -> Option<Arc<SourceFile>> {
    None
}

pub struct NeedsAutoImports;

pub const NEEDS_AUTO_IMPORTS: NeedsAutoImports = NeedsAutoImports;
