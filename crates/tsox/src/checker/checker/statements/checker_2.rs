#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn declaration_is_ambient(&self, node: &Arc<Node>) -> bool {
        if self.ambient_context_depth > 0 {
            return true;
        }

        if self
            .get_source_file_of_node(node)
            .or(self.current_file.clone())
            .is_some_and(|f| f.is_declaration_file)
        {
            return true;
        }
        let mut cur = Some(node);
        while let Some(n) = cur {
            if n.has_syntactic_modifier(ModifierFlags::Ambient) {
                return true;
            }

            if matches!(
                n.kind,
                SyntaxKind::VariableStatement
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::FunctionDeclaration
            ) {
                break;
            }
            cur = n.parent.as_ref();
        }
        false
    }

    pub(crate) fn check_cjs_reserved_top_level_name(&mut self, node: &Arc<Node>, name: &Arc<Node>) {
        use crate::core::compiler_options::ModuleKind;
        if !matches!(name.kind, SyntaxKind::Identifier) {
            return;
        }

        if self.compiler_options.no_emit.is_true() {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        let is_module =
            file.external_module_indicator.is_some() || file.common_js_module_indicator.is_some();
        if !is_module {
            return;
        }

        let mut top_level = false;
        let mut p = node.parent.as_ref();
        while let Some(parent) = p {
            match parent.kind {
                SyntaxKind::SourceFile => {
                    top_level = true;
                    break;
                }
                SyntaxKind::VariableDeclarationList | SyntaxKind::VariableStatement => {
                    p = parent.parent.as_ref();
                }
                _ => break,
            }
        }
        if !top_level {
            return;
        }

        if self.declaration_is_ambient(node) {
            return;
        }
        let emit_format = self.program.get_emit_module_format_of_file(&file.file_name);
        let text = name.text().to_string();
        if text == "require" || text == "exports" {
            if emit_format >= ModuleKind::ES2015 {
                return;
            }
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    DUPLICATE_IDENTIFIER_0_COMPILER_RESERVES_NAME_1_IN_TOP_LEVEL_SCOPE_OF_A_MODULE,
                vec![text.clone(), text],
            ));
        } else if text == "__esModule" {
            let var_stmt = node
                .parent
                .as_ref()
                .and_then(|list| list.parent.as_ref())
                .filter(|stmt| stmt.kind == SyntaxKind::VariableStatement);
            let exported =
                var_stmt.is_some_and(|stmt| stmt.has_syntactic_modifier(ModifierFlags::Export));
            if !exported || emit_format >= ModuleKind::System {
                return;
            }
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    IDENTIFIER_EXPECTED_ESMODULE_IS_RESERVED_AS_AN_EXPORTED_MARKER_WHEN_TRANSFORMING_ECMASCRIPT_MODULES,
                Vec::new(),
            ));
        } else if text == "Object" && node.kind == SyntaxKind::ClassDeclaration {
            if emit_format != ModuleKind::CommonJS {
                return;
            }
            let module_str = match self.compiler_options.module {
                ModuleKind::Node16 => "Node16".to_string(),
                ModuleKind::Node18 => "Node18".to_string(),
                ModuleKind::Node20 => "Node20".to_string(),
                ModuleKind::NodeNext => "NodeNext".to_string(),
                ModuleKind::CommonJS => "CommonJS".to_string(),
                ModuleKind::AMD => "AMD".to_string(),
                ModuleKind::UMD => "UMD".to_string(),
                ModuleKind::System => "System".to_string(),
                ModuleKind::ES2015 => "es2015".to_string(),
                ModuleKind::ES2020 => "es2020".to_string(),
                ModuleKind::ES2022 => "es2022".to_string(),
                _ => "esnext".to_string(),
            };
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    CLASS_NAME_CANNOT_BE_OBJECT_WHEN_TARGETING_ES5_AND_ABOVE_WITH_MODULE_0,
                vec![module_str],
            ));
        }
    }

    pub(crate) fn check_for_of_iterated_type(
        &mut self,
        statement: &Arc<Node>,
        expression: &Arc<Node>,
    ) {
        let readonly_array_exists = match self.globals.get("ReadonlyArray") {
            Some(sym) => !sym.members.is_empty(),
            None => false,
        };
        if !readonly_array_exists {
            return;
        }
        let t = self.get_type_of_node(expression);
        if t.flags.contains(TypeFlags::Any | TypeFlags::Never) {
            return;
        }
        let mut parts: Vec<Arc<Type>> = Vec::new();
        if t.is_union() {
            parts = self.constituent_types(&t);
        } else {
            parts.push(t.clone());
        }
        for part in &parts {
            let is_string_like = part
                .flags
                .intersects(TypeFlags::String | TypeFlags::StringLiteral);
            if !(self.is_array_type(part) || self.is_tuple_type(part) || is_string_like) {
                let type_str = self.type_to_string(&t);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    expression.loc,
                    crate::diagnostics::messages_generated::
                        TYPE_0_IS_NOT_AN_ARRAY_TYPE_OR_A_STRING_TYPE,
                    vec![type_str],
                ));
                return;
            }
        }
        let _ = statement;
    }

    pub(crate) fn check_for_initializer(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::VariableDeclarationList => {
                self.check_variable_declaration_list(node);
            }
            _ => self.check_expression(node),
        }
    }

    pub(crate) fn check_binding_pattern_computed_names(&mut self, name: &Arc<Node>) {
        if !matches!(
            name.kind,
            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
        ) {
            return;
        }
        let mut stack = vec![Arc::clone(name)];
        while let Some(n) = stack.pop() {
            match &n.data {
                crate::ast::NodeData::BindingPattern(data) => {
                    for element in data.elements.iter() {
                        stack.push(Arc::clone(element));
                    }
                }
                crate::ast::NodeData::BindingElement(data) => {
                    if let Some(pn) = &data.property_name {
                        if pn.kind == SyntaxKind::ComputedPropertyName {
                            self.check_computed_property_name(pn);
                            if let crate::ast::NodeData::ComputedPropertyName(cd) = &pn.data {
                                self.check_expression(&cd.expression);

                                let expr_type = self.get_type_of_node(&cd.expression);
                                let is_any = match &expr_type.data {
                                    crate::checker::types::TypeData::Union(u) => u
                                        .union_or_intersection
                                        .types
                                        .iter()
                                        .any(|t| t.flags.contains(TypeFlags::Any)),
                                    _ => expr_type.flags.contains(TypeFlags::Any),
                                };
                                if is_any {
                                    let file = self.current_file.clone();
                                    let type_str = self.type_to_string(&expr_type);
                                    let diagnostic = crate::ast::Diagnostic::new(
                                        file,
                                        cd.expression.loc,
                                        crate::diagnostics::messages_generated::
                                            TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                                        vec![type_str],
                                    );
                                    self.diagnostics.add(diagnostic);
                                }
                            }
                        }
                    }
                    if let Some(inner) = &data.name {
                        if matches!(
                            inner.kind,
                            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
                        ) {
                            stack.push(Arc::clone(inner));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
