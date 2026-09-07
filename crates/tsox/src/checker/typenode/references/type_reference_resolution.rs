#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn resolve_type_parameter_reference(
        &mut self,
        symbol: &Arc<Symbol>,
        type_name: &Arc<Node>,
    ) -> Arc<Type> {
        if self.in_static_member_type {
            let tp_decl = symbol
                .value_declaration
                .clone()
                .or_else(|| symbol.declarations.first().cloned());
            let owned_by_class = tp_decl.is_some_and(|d| {
                let mut cur = d.parent.as_ref();
                while let Some(a) = cur {
                    match a.kind {
                        crate::ast::SyntaxKind::ClassDeclaration
                        | crate::ast::SyntaxKind::ClassExpression => return true,
                        crate::ast::SyntaxKind::SourceFile => return false,
                        _ => cur = a.parent.as_ref(),
                    }
                }
                false
            });
            if owned_by_class {
                use crate::diagnostics::messages_generated::STATIC_MEMBERS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS;
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    STATIC_MEMBERS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS,
                    Vec::new(),
                ));
            }
        }

        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        for map in self.type_argument_stack.iter().rev() {
            if let Some(t) = map.get(&key) {
                return Arc::clone(t);
            }
        }

        for frame in self.type_argument_name_frames.iter().rev() {
            for (frame_sym, t) in frame.iter().rev() {
                if Arc::ptr_eq(frame_sym, symbol)
                    || (frame_sym.name == symbol.name
                        && self.type_param_symbols_share_container(frame_sym, symbol))
                {
                    return Arc::clone(t);
                }
            }
        }
        return self.get_type_parameter_from_symbol(symbol);
    }

    pub(crate) fn resolve_type_alias_reference(
        &mut self,
        symbol: &Arc<Symbol>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Type> {
        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            return self.error_type();
        }

        let has_type_args = type_arguments.is_some();
        let resolved = if !has_type_args {
            let cached = self
                .type_alias_links
                .get(&symbol)
                .and_then(|l| l.declared_type.clone());
            cached.unwrap_or_else(|| {
                let saved_static = self.in_static_member_type;
                self.in_static_member_type = false;
                let found = self.resolve_alias_body(symbol);
                self.in_static_member_type = saved_static;
                self.type_alias_links.get_or_default(symbol).declared_type =
                    Some(Arc::clone(&found));
                found
            })
        } else {
            let (tp_symbols, type_node) = self.collect_alias_type_params_and_body(symbol);
            let arg_types: Vec<Arc<Type>> = match &type_arguments {
                Some(args) => args
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect(),
                None => Vec::new(),
            };
            let mut mapping = HashMap::new();
            for (i, tp_sym) in tp_symbols.iter().enumerate() {
                if i < arg_types.len() {
                    let tp_key = Arc::as_ptr(tp_sym) as *const crate::ast::Symbol;
                    mapping.insert(tp_key, Arc::clone(&arg_types[i]));
                }
            }
            self.type_argument_stack.push(mapping);

            let alias_decl = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::TypeAliasDeclaration)
                .cloned();
            if let Some(decl) = &alias_decl {
                self.push_scope(decl);
            }
            let saved_static = self.in_static_member_type;
            self.in_static_member_type = false;
            let found = self.get_type_from_type_node(&type_node);
            self.in_static_member_type = saved_static;
            if alias_decl.is_some() {
                self.pop_scope();
            }
            self.type_argument_stack.pop();
            found
        };
        self.pop_type_resolution();
        resolved
    }
}
