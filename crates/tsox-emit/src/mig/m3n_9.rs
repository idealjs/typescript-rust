use super::m3n_5::create_diagnostic_for_node;
use super::m3n_7::{DeclarationTransformer, ReferencedFilePair};
use crate::mig::m3m_2::create_get_symbol_accessibility_diagnostic_for_node;
use crate::mig::m4e::{R37K1DataExt, R39K01DataExt};
use std::sync::Arc;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics::messages_generated as diag_msgs;
use tsox_core::tspath::{get_relative_path_to_directory_or_url, ComparePathsOptions};
use super::m3n_5::r33k8_defs::FileReference;
use tsox_frontend::ast::mig::x4ast::get_external_module_import_equals_declaration_expression;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

impl DeclarationTransformer {
    fn resolver_is_declaration_visible(&self, node: &Arc<Node>) -> bool {
        crate::mig::m4e::r39k01_defs::R39K01EmitResolverExt::is_declaration_visible(
            &self.resolver,
            node,
        )
    }

    fn resolver_is_import_required_by_augmentation(&self, decl: &Arc<Node>) -> bool {
        crate::mig::m4e::r39k01_defs::R39K01EmitResolverExt::is_import_required_by_augmentation(
            &self.resolver,
            decl,
        )
    }

    pub fn transform_import_equals_declaration(&mut self, decl: &Arc<Node>) -> Option<Arc<Node>> {
        if !self.resolver_is_declaration_visible(decl) {
            return None;
        }
        let ied = decl.as_import_equals_declaration();
        if ied.module_reference.kind == SyntaxKind::ExternalModuleReference {
            let specifier = get_external_module_import_equals_declaration_expression(decl);
            let rewritten = self.rewrite_module_specifier(decl, Some(&specifier));
            let rewritten = rewritten.unwrap_or_else(|| {
                ied.module_reference
                    .as_external_module_reference()
                    .expression
                    .clone()
            });
            Some(self.factory().update_import_equals_declaration(
                decl,
                decl.modifiers().cloned(),
                ied.is_type_only,
                decl.name().unwrap().clone(),
                self.factory()
                    .update_external_module_reference(&ied.module_reference, rewritten),
            ))
        } else {
            let old_diag = self.state.get_symbol_accessibility_diagnostic.take();
            self.state.get_symbol_accessibility_diagnostic =
                Some(create_get_symbol_accessibility_diagnostic_for_node(decl));
            let module_reference = ied.module_reference.clone();
            let enclosing = self.enclosing_declaration.clone();
            self.check_entity_name_visibility(&module_reference, enclosing.as_ref());
            self.state.get_symbol_accessibility_diagnostic = old_diag;
            Some(decl.clone())
        }
    }

    pub fn transform_import_declaration(&mut self, decl: &Arc<Node>) -> Option<Arc<Node>> {
        let d = decl.as_import_declaration();
        if d.import_clause.is_none() {
            let module_specifier = self
                .rewrite_module_specifier(decl, Some(&d.module_specifier))
                .unwrap_or_else(|| d.module_specifier.clone());
            return Some(self.factory().update_import_declaration(
                decl,
                decl.modifiers().cloned(),
                d.import_clause.clone(),
                module_specifier,
                d.attributes.clone(),
            ));
        }
        let import_clause = d.import_clause.clone().unwrap();
        let ic = import_clause.as_import_clause();
        let mut phase_modifier = ic.phase_modifier.unwrap_or(SyntaxKind::Unknown);
        if phase_modifier == SyntaxKind::DeferKeyword {
            phase_modifier = SyntaxKind::Unknown;
        }
        let phase_modifier = if phase_modifier == SyntaxKind::Unknown {
            None
        } else {
            Some(phase_modifier)
        };
        let mut visible_default_binding: Option<Arc<Node>> = None;
        if import_clause.name().is_some() && self.resolver_is_declaration_visible(&import_clause) {
            visible_default_binding = import_clause.name().cloned();
        }
        let named_bindings_empty = ic.named_bindings.is_none();
        if named_bindings_empty {
            if visible_default_binding.is_none() {
                return None;
            }
            let module_specifier = self
                .rewrite_module_specifier(decl, Some(&d.module_specifier))
                .unwrap_or_else(|| d.module_specifier.clone());
            return Some(self.factory().update_import_declaration(
                decl,
                decl.modifiers().cloned(),
                Some(self.factory().update_import_clause(
                    &import_clause,
                    phase_modifier,
                    visible_default_binding,
                    None,
                )),
                module_specifier,
                d.attributes.clone(),
            ));
        }
        let named_bindings = ic.named_bindings.clone().unwrap();
        if named_bindings.kind == SyntaxKind::NamespaceImport {
            let mut new_named_bindings: Option<Arc<Node>> = None;
            if self.resolver_is_declaration_visible(&named_bindings) {
                new_named_bindings = Some(named_bindings.clone());
            }
            if visible_default_binding.is_none() && new_named_bindings.is_none() {
                return None;
            }
            let module_specifier = self
                .rewrite_module_specifier(decl, Some(&d.module_specifier))
                .unwrap_or_else(|| d.module_specifier.clone());
            return Some(self.factory().update_import_declaration(
                decl,
                decl.modifiers().cloned(),
                Some(self.factory().update_import_clause(
                    &import_clause,
                    phase_modifier,
                    visible_default_binding,
                    new_named_bindings,
                )),
                module_specifier,
                d.attributes.clone(),
            ));
        }
        let named_import_elements: Vec<Arc<Node>> = match &named_bindings.data {
            tsox_frontend::ast::NodeData::NamedImports(elements) => elements.elements.nodes.clone(),
            _ => Vec::new(),
        };
        let binding_list: Vec<Arc<Node>> = named_import_elements
            .iter()
            .filter(|b| self.resolver_is_declaration_visible(b))
            .cloned()
            .collect();
        if !binding_list.is_empty() || visible_default_binding.is_some() {
            let mut named_imports: Option<Arc<Node>> = None;
            if !binding_list.is_empty() {
                named_imports = Some(self.factory().update_named_imports(
                    &named_bindings,
                    &self.factory().new_node_list(binding_list),
                ));
            }
            let module_specifier = self
                .rewrite_module_specifier(decl, Some(&d.module_specifier))
                .unwrap_or_else(|| d.module_specifier.clone());
            return Some(self.factory().update_import_declaration(
                decl,
                decl.modifiers().cloned(),
                Some(self.factory().update_import_clause(
                    &import_clause,
                    phase_modifier,
                    visible_default_binding,
                    named_imports,
                )),
                module_specifier,
                d.attributes.clone(),
            ));
        }
        if self.resolver_is_import_required_by_augmentation(decl) {
            if self.state.isolated_declarations {
                self.state.add_diagnostic(create_diagnostic_for_node(
                    decl,
                    Some(&diag_msgs::DECLARATION_EMIT_FOR_THIS_FILE_REQUIRES_PRESERVING_THIS_IMPORT_FOR_AUGMENTATIONS_THIS_IS_NOT_SUPPORTED_WITH_ISOLATEDDECLARATIONS),
                    &[],
                ));
            }
            let module_specifier = self
                .rewrite_module_specifier(decl, Some(&d.module_specifier))
                .unwrap_or_else(|| d.module_specifier.clone());
            return Some(self.factory().update_import_declaration(
                decl,
                decl.modifiers().cloned(),
                None,
                module_specifier,
                d.attributes.clone(),
            ));
        }
        None
    }
}

pub fn get_referenced_files(
    tx: &DeclarationTransformer,
    output_file_path: &str,
) -> Vec<FileReference> {
    let mut results = Vec::new();
    for ReferencedFilePair {
        file: source_file,
        reference,
    } in &tx.raw_referenced_files
    {
        if !reference.preserve {
            continue;
        }
        let Some(file) = tx
            .host
            .get_source_file_from_reference(source_file, reference)
        else {
            continue;
        };
        let decl_file_name = if file.is_declaration_file {
            file.file_name.clone()
        } else {
            let paths = tx.host.get_output_paths_for(&file, true);
            let mut name = paths.declaration_file_path();
            if name.is_empty() {
                name = paths.js_file_path();
            }
            if name.is_empty() {
                name = file.file_name.clone();
            }
            name
        };
        if decl_file_name.is_empty() {
            continue;
        }
        let file_name = get_relative_path_to_directory_or_url(
            output_file_path,
            &decl_file_name,
            false,
            &ComparePathsOptions {
                current_directory: tx.host.get_current_directory(),
                use_case_sensitive_file_names: tx.host.use_case_sensitive_file_names(),
            },
        );
        results.push(FileReference {
            text_range: TextRange::new(usize::MAX, usize::MAX),
            file_name,
            resolution_mode: reference.resolution_mode,
            preserve: reference.preserve,
        });
    }
    results
}

pub fn get_lib_references(tx: &DeclarationTransformer) -> Vec<FileReference> {
    tx.raw_lib_reference_directives
        .iter()
        .filter(|r| r.preserve)
        .map(|r| FileReference {
            text_range: TextRange::new(usize::MAX, usize::MAX),
            file_name: r.file_name.clone(),
            resolution_mode: r.resolution_mode,
            preserve: r.preserve,
        })
        .collect()
}

pub fn get_type_references(tx: &DeclarationTransformer) -> Vec<FileReference> {
    tx.raw_type_reference_directives
        .iter()
        .filter(|r| r.preserve)
        .map(|r| FileReference {
            text_range: TextRange::new(usize::MAX, usize::MAX),
            file_name: r.file_name.clone(),
            resolution_mode: r.resolution_mode,
            preserve: r.preserve,
        })
        .collect()
}
