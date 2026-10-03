use crate::binder::nameresolver::NameResolver;
use crate::binder::*;
use std::sync::Arc;
use super::r19k5_ext::{
    export_assignment_is_export_equals, export_specifier_name, set_symbol_flags_or,
    SourceFileNodeExt, SymbolFlagsExt,
};
use super::r19k5_msg as msg;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::mig::m3g_3::module_export_name_is_default;
use tsox_frontend::ast::*;
use tsox_frontend::scanner::mig::m4d_2::get_range_of_token_at_position;

impl Binder {
    pub(crate) fn declare_module_symbol(&mut self, node: &Arc<Node>) -> ModuleInstanceState { ::tsox_core::fntrace::enter("declare_module_symbol"); 
        let state = get_module_instance_state(node);
        let instantiated = state != ModuleInstanceState::NonInstantiated;
        let (includes, excludes) = if instantiated {
            (SymbolFlags::ValueModule, SymbolFlags::ValueModuleExcludes)
        } else {
            (
                SymbolFlags::NamespaceModule,
                SymbolFlags::NamespaceModuleExcludes,
            )
        };
        self.declare_symbol_and_add_to_symbol_table(node, includes, excludes);
        state
    }

    pub(crate) fn declare_source_file_member(
        &mut self,
        node: &Arc<Node>,
        symbol_flags: SymbolFlags,
        symbol_excludes: SymbolFlags,
    ) -> Arc<Symbol> { ::tsox_core::fntrace::enter("declare_source_file_member"); 
        let file = self.current_source_file.clone().unwrap();
        if is_external_module(&file) {
            self.declare_module_member(node, symbol_flags, symbol_excludes)
        } else {
            self.declare_symbol_into(
                node,
                symbol_flags,
                symbol_excludes,
                DeclareTarget::Locals(file.node.clone()),
            )
        }
    }

    pub(crate) fn declare_symbol_and_add_to_symbol_table(
        &mut self,
        node: &Arc<Node>,
        symbol_flags: SymbolFlags,
        symbol_excludes: SymbolFlags,
    ) -> Arc<Symbol> { ::tsox_core::fntrace::enter("declare_symbol_and_add_to_symbol_table"); 
        let container_kind = self.container.as_ref().unwrap().kind;
        match container_kind {
            SyntaxKind::ModuleDeclaration => {
                self.declare_module_member(node, symbol_flags, symbol_excludes)
            }
            SyntaxKind::SourceFile => {
                self.declare_source_file_member(node, symbol_flags, symbol_excludes)
            }
            SyntaxKind::ClassExpression | SyntaxKind::ClassDeclaration => {
                self.declare_class_member(node, symbol_flags, symbol_excludes)
            }
            SyntaxKind::EnumDeclaration => {
                let container = self.container.clone().unwrap();
                let enum_symbol = self.symbol_map.symbol_of(&container).cloned().unwrap();
                self.declare_symbol_into(
                    node,
                    symbol_flags,
                    symbol_excludes,
                    DeclareTarget::Exports(enum_symbol),
                )
            }
            SyntaxKind::TypeLiteral
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::JsxAttributes => {
                let container = self.container.clone().unwrap();
                let container_symbol = self.symbol_map.symbol_of(&container).cloned().unwrap();
                self.declare_symbol_into(
                    node,
                    symbol_flags,
                    symbol_excludes,
                    DeclareTarget::Members(container_symbol),
                )
            }
            SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::MappedType => {
                let container = self.container.clone().unwrap();
                self.declare_symbol_into(
                    node,
                    symbol_flags,
                    symbol_excludes,
                    DeclareTarget::Locals(container),
                )
            }
            _ => panic!("Unhandled case in declareSymbolAndAddToSymbolTable"),
        }
    }

    pub(crate) fn declare_symbol_ex(
        &mut self,
        symbol_table: &mut SymbolTable,
        parent: Option<&Arc<Symbol>>,
        node: &Arc<Node>,
        includes: SymbolFlags,
        excludes: SymbolFlags,
        is_replaceable_by_method: bool,
        is_computed_name: bool,
    ) -> Arc<Symbol> { ::tsox_core::fntrace::enter("declare_symbol_ex"); 
        let is_default_export = has_syntactic_modifier(node, ModifierFlags::Default)
            || is_export_specifier(node)
                && module_export_name_is_default(&export_specifier_name(node));
        let name: String = if is_computed_name {
            "___computed".to_string()
        } else if is_default_export && parent.is_some() {
            "default".to_string()
        } else {
            self.get_declaration_name(node)
        };
        let mut symbol: Option<Arc<Symbol>>;
        if name == "___missing" {
            symbol = Some(self.new_symbol(SymbolFlags::empty(), "___missing".to_string()));
        } else {
            let existing = symbol_table.get(&name).cloned();
            if existing.is_none() {
                let created = self.new_symbol(SymbolFlags::empty(), name.clone());
                symbol_table.insert(name.clone(), Arc::clone(&created));
                if is_replaceable_by_method {
                    set_symbol_flags_or(&created, SymbolFlags::ReplaceableByMethod);
                }
                symbol = Some(created);
            } else if is_replaceable_by_method
                && !existing.as_ref().unwrap().flags.contains(SymbolFlags::ReplaceableByMethod)
            {
                return existing.unwrap();
            } else if existing.as_ref().unwrap().flags.intersects(excludes) {
                let mut current = existing.unwrap();
                if current.flags.contains(SymbolFlags::ReplaceableByMethod) {
                    let created = self.new_symbol(SymbolFlags::empty(), name.clone());
                    symbol_table.insert(name.clone(), Arc::clone(&created));
                    current = created;
                } else if !(includes.contains(SymbolFlags::Variable)
                    && current.flags.contains(SymbolFlags::Assignment)
                    || includes.contains(SymbolFlags::Assignment)
                        && current.flags.contains(SymbolFlags::Variable))
                {
                    let mut message: &'static Message =
                        if current.flags.contains(SymbolFlags::BlockScopedVariable)
                        {
                            &msg::Cannot_redeclare_block_scoped_variable_0
                        } else {
                            &msg::Duplicate_identifier_0
                        };
                    let mut message_needs_name = true;
                    if current.flags.contains(SymbolFlags::Enum) || includes.contains(SymbolFlags::Enum)
                    {
                        message =
                            &msg::Enum_declarations_can_only_merge_with_namespace_or_other_enum_declarations;
                        message_needs_name = false;
                    }
                    let mut multiple_default_exports = false;
                    if !current.declarations.is_empty() && is_default_export {
                        message = &msg::A_module_cannot_have_multiple_default_exports;
                        message_needs_name = false;
                        multiple_default_exports = true;
                    } else if !current.declarations.is_empty()
                        && is_export_assignment(node)
                        && !export_assignment_is_export_equals(node)
                    {
                        message = &msg::A_module_cannot_have_multiple_default_exports;
                        message_needs_name = false;
                        multiple_default_exports = true;
                    }
                    let declaration_name = get_name_of_declaration(node).unwrap_or_else(|| Arc::clone(node));
                    let diag = if message_needs_name {
                        self.create_diagnostic_for_node(
                            &declaration_name,
                            message,
                            &[self.get_display_name(node)],
                        )
                    } else {
                        self.create_diagnostic_for_node(&declaration_name, message, &[])
                    };
                    for (index, declaration) in current.declarations.iter().enumerate() {
                        let decl =
                            get_name_of_declaration(declaration).unwrap_or_else(|| Arc::clone(declaration));
                        let d = if message_needs_name {
                            self.create_diagnostic_for_node(
                                &decl,
                                message,
                                &[self.get_display_name(declaration)],
                            )
                        } else {
                            self.create_diagnostic_for_node(&decl, message, &[])
                        };
                        self.symbol_map.binder_diagnostics.push(d.clone());
                        if multiple_default_exports {
                            let related = self.create_diagnostic_for_node(
                                &declaration_name,
                                if index == 0 {
                                    &msg::Another_export_default_is_here
                                } else {
                                    &msg::X_and_here
                                },
                                &[],
                            );
                            if let Some(last) = self.symbol_map.binder_diagnostics.last_mut() {
                                last.related_information.push(related);
                            }
                        }
                    }
                    self.symbol_map.binder_diagnostics.push(diag);
                    if current.flags.contains(SymbolFlags::Accessor)
                        && !includes.contains(SymbolFlags::Accessor)
                    {
                        set_symbol_flags_or(&current, SymbolFlags::Accessor);
                    }
                    current = self.new_symbol(SymbolFlags::empty(), name.clone());
                }
                symbol = Some(current);
            } else {
                symbol = existing;
            }
        }
        let symbol = symbol.unwrap();
        self.add_declaration_to_symbol(&symbol, node, includes);
        if symbol.parent().is_none() {
            if let Some(p) = parent {
                symbol.set_parent(p);
            }
        }
        symbol
    }

    pub(crate) fn error_on_first_token(
        &mut self,
        node: &Arc<Node>,
        message: &'static Message,
        args: &[String],
    ) { ::tsox_core::fntrace::enter("error_on_first_token"); 
        let file = self.current_source_file.clone().unwrap();
        let span = get_range_of_token_at_position(&file, node.pos());
        self.symbol_map
            .binder_diagnostics
            .push(Diagnostic::new(Some(file), span, *message, args.to_vec()));
    }

    pub(crate) fn error_on_node(
        &mut self,
        node: &Arc<Node>,
        message: &'static Message,
        args: &[String],
    ) { ::tsox_core::fntrace::enter("error_on_node"); 
        let diag = self.create_diagnostic_for_node(node, message, args);
        self.symbol_map.binder_diagnostics.push(diag);
    }
}
