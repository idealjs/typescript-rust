use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::ast::node_data_generated::NodeData;
use crate::ast::{
    CheckFlags, ModifierFlags, Node, NodeList, Symbol, SymbolFlags, SymbolTable,
    SyntaxKind,
};
use crate::jsnum;

use crate::checker::checker::Checker;


use super::*;


impl Checker {
    pub(crate) fn get_type_from_this_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn get_type_from_literal_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let literal = match &node.data {
            NodeData::LiteralTypeNode(data) => &data.literal,
            _ => return self.error_type(),
        };
        if literal.kind == SyntaxKind::NullKeyword {
            return self.null_type();
        }
        let result = self.literal_type_from_literal_node(literal);
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn literal_type_from_literal_node(&mut self, literal: &Arc<Node>) -> Arc<Type> {
        match literal.kind {
            SyntaxKind::StringLiteral => self.get_string_literal_type(literal.text()),
            SyntaxKind::NumericLiteral => {
                if let Ok(n) = literal.text().parse::<f64>() {
                    self.get_number_literal_type(crate::jsnum::Number::from(n))
                } else {
                    self.number_type()
                }
            }
            SyntaxKind::BigIntLiteral => {
                let text = literal.text();
                if let Some(t) = self.bigint_literal_types.get(text).cloned() {
                    return t;
                }
                let (neg, digits) = if let Some(rest) = text.strip_prefix('-') {
                    (true, rest.trim_end_matches('n'))
                } else {
                    (false, text.trim_end_matches('n'))
                };
                let t = Arc::new(Type::new(
                    TypeFlags::BigIntLiteral,
                    TypeData::Literal(LiteralTypeData {
                        value: LiteralValue::BigInt(crate::jsnum::PseudoBigInt::new(digits, neg)),
                        fresh_type: std::sync::OnceLock::new(),
                        regular_type: std::sync::OnceLock::new(),
                    }),
                ));
                self.bigint_literal_types
                    .insert(text.to_string(), Arc::clone(&t));
                t
            }
            SyntaxKind::TrueKeyword => self.true_type(),
            SyntaxKind::FalseKeyword => self.false_type(),
            _ => self.error_type(),
        }
    }

    pub(crate) fn get_type_from_type_reference(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.resolve_type_reference(node);
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn resolve_type_reference(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (type_name, type_arguments) = match &node.data {
            NodeData::TypeReferenceNode(data) => (&data.type_name, data.type_arguments.clone()),
            NodeData::ExpressionWithTypeArguments(data) => {
                (&data.expression, data.type_arguments.clone())
            }
            _ => return self.error_type(),
        };

        if type_name.kind == SyntaxKind::Identifier && type_name.text() == "intrinsic" {
            return self.error_type();
        }
        let mut symbol = if type_name.kind == SyntaxKind::Identifier {
            match self.resolve_identifier(type_name) {
                Some(s) => s,
                None => {

                    if self.ts2304_reporting_allowed_for(type_name) {
                        use crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0;
                        let name_text = type_name.text();

                        let file = self
                            .get_source_file_of_node(type_name)
                            .or_else(|| self.current_file.clone());
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            type_name.loc,
                            CANNOT_FIND_NAME_0,
                            vec![name_text.to_string()],
                        ));
                    }
                    return self.error_type();
                }
            }
        } else if matches!(
            type_name.kind,
            SyntaxKind::Identifier
                | SyntaxKind::QualifiedName

                | SyntaxKind::PropertyAccessExpression
        ) {
            match self.resolve_qualified_symbol_traced(type_name) {
                Ok(s) => s,
                Err((segment, ns_path, member)) => {

                    let attributed_file = self
                        .get_source_file_of_node(type_name)
                        .or_else(|| self.current_file.clone());
                    let reportable = type_name.kind == SyntaxKind::QualifiedName;
                    if reportable
                        && self.ts2304_reporting_allowed_for(type_name)
                        && attributed_file
                            .as_ref()
                            .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                    {
                        let file = attributed_file;
                        if ns_path.is_empty() {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                segment.loc,
                                crate::diagnostics::messages_generated::
                                    CANNOT_FIND_NAMESPACE_0,
                                vec![segment.text().to_string()],
                            ));
                        } else {

                            let leftmost = crate::checker::checker::base_identifier_of(type_name);
                            let left_hit = self
                                .resolve_identifier(&leftmost)
                                .map(|s| self.resolve_alias_base(s));
                            let left_non_namespace = left_hit
                                .as_ref()
                                .is_some_and(|b| !b.flags.intersects(SymbolFlags::NAMESPACE));
                            if left_non_namespace {
                                let name_text = leftmost.text().to_string();
                                if let Some(sugg) =
                                    self.find_name_suggestion(&name_text, SymbolFlags::NAMESPACE)
                                {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        leftmost.loc,
                                        crate::diagnostics::messages_generated::
                                            CANNOT_FIND_NAMESPACE_0_DID_YOU_MEAN_1,
                                        vec![name_text.clone(), sugg],
                                    ));
                                } else if left_hit
                                    .as_ref()
                                    .is_some_and(|b| b.flags.intersects(SymbolFlags::TYPE))
                                {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        leftmost.loc,
                                        crate::diagnostics::messages_generated::
                                            X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_NAMESPACE_HERE,
                                        vec![name_text],
                                    ));
                                } else {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        leftmost.loc,
                                        crate::diagnostics::messages_generated::
                                            CANNOT_FIND_NAMESPACE_0,
                                        vec![name_text],
                                    ));
                                }
                            } else {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    segment.loc,
                                    crate::diagnostics::messages_generated::
                                        NAMESPACE_0_HAS_NO_EXPORTED_MEMBER_1,
                                    vec![ns_path, member],
                                ));
                            }
                        }
                    }
                    return self.error_type();
                }
            }
        } else {
            return self.error_type();
        };

        if symbol.flags == SymbolFlags::Alias {
            let alias_name = type_name.text().to_string();
            if let Some(target) = self.resolve_import_alias_target_symbol(&symbol) {
                let target_has_type_meaning = target.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeAlias
                        | SymbolFlags::ENUM
                        | SymbolFlags::TypeParameter,
                );
                if target_has_type_meaning {
                    symbol = target;
                } else {
                    if type_name.kind == SyntaxKind::Identifier
                        && self.ts2304_reporting_allowed_for(type_name)
                        && !self.has_same_named_type_symbol(&alias_name)
                        && self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                    {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            type_name.loc,
                            crate::diagnostics::messages_generated::
                                X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                            vec![alias_name.clone(), alias_name],
                        ));
                    }
                    return self.error_type();
                }
            }
        }

        if !self
            .current_file
            .as_ref()
            .is_some_and(|f| f.file_name.starts_with("bundled://"))
            && symbol
                .flags
                .intersects(SymbolFlags::Interface | SymbolFlags::Class | SymbolFlags::TypeAlias)
            && !type_name_inside_conditional_branch(type_name)
            && !type_name_shadowed_by_type_parameter(type_name)
            && !self.check_type_reference_arguments(node, type_name, &symbol)
        {

            return self.error_type();
        }

        if symbol.flags.contains(SymbolFlags::TypeParameter) {

            if self.in_static_member_type {

                let tp_decl = symbol.value_declaration.clone().or_else(|| {
                    symbol.declarations.first().cloned()
                });
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

            let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
            for map in self.type_argument_stack.iter().rev() {
                if let Some(t) = map.get(&key) {
                    return Arc::clone(t);
                }
            }

            for frame in self.type_argument_name_frames.iter().rev() {
                for (frame_sym, t) in frame.iter().rev() {
                    if Arc::ptr_eq(frame_sym, &symbol)
                        || (frame_sym.name == symbol.name
                            && self.type_param_symbols_share_container(frame_sym, &symbol))
                    {
                        return Arc::clone(t);
                    }
                }
            }
            return self.get_type_parameter_from_symbol(&symbol);
        }
        if symbol.flags.contains(SymbolFlags::Interface) {

            return self.resolve_interface_type(&symbol, type_arguments);
        }
        if symbol.flags.intersects(SymbolFlags::ENUM) {

            return self.resolve_enum_type(&symbol);
        }
        if symbol.flags.contains(SymbolFlags::Class) {

            let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
            let merged_with_ns = symbol.flags.contains(SymbolFlags::ValueModule);
            if !merged_with_ns {
                if let Some(cached) = self
                    .type_alias_links
                    .get(&symbol)
                    .and_then(|l| l.declared_type.clone())
                {
                    return cached;
                }
            }
            if !self.resolving_type_aliases.insert(key) {
                return self.error_type();
            }
            let class_node = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                .cloned();
            let instance_type = match class_node {
                Some(node) => self.build_class_instance_type_with_base(&node),
                None => self.error_type(),
            };
            self.resolving_type_aliases.remove(&key);
            if !merged_with_ns {
                self.type_alias_links.get_or_default(&symbol).declared_type =
                    Some(Arc::clone(&instance_type));
            }

            let arg_types: Option<Vec<Arc<Type>>> = type_arguments.map(|nodes| {
                nodes
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect()
            });
            if let Some(arg_types) = arg_types {
                let tps = self.declared_type_parameter_types(&symbol);
                if !tps.is_empty() && tps.len() == arg_types.len() {
                    return self.attach_explicit_type_arguments_cached(&instance_type, arg_types);
                }
            }
            return instance_type;
        }
        if !symbol.flags.contains(SymbolFlags::TypeAlias) {

            if matches!(&node.data, NodeData::ExpressionWithTypeArguments(_))
                && symbol.flags.intersects(
                    SymbolFlags::BlockScopedVariable
                        | SymbolFlags::FunctionScopedVariable
                        | SymbolFlags::Function,
                )
                && !symbol.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeAlias
                        | SymbolFlags::TypeParameter,
                )
            {
                let value_type = self.get_type_of_symbol(&symbol);
                if let Some(structured) = value_type.as_structured() {
                    for sig in structured.construct_signatures() {
                        if let Some(ret) = self.get_return_type_of_signature(sig) {
                            return ret;
                        }
                    }
                }
                return self.get_any_type();
            }

            if type_name.kind == SyntaxKind::Identifier
                && symbol.flags.intersects(
                    SymbolFlags::BlockScopedVariable
                        | SymbolFlags::FunctionScopedVariable
                        | SymbolFlags::Function,
                )
                && !symbol.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeParameter
                        | SymbolFlags::TypeAlias

                        | SymbolFlags::Alias,
                )
                && self.ts2304_reporting_allowed_for(type_name)
                && !self.has_same_named_type_symbol(type_name.text())

                && self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| !f.file_name.starts_with("bundled://"))
            {
                let name_text = type_name.text().to_string();
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    crate::diagnostics::messages_generated::
                        X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                    vec![name_text.clone(), name_text],
                ));
            }

            return self.error_type();
        }

        let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
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
                let found = self.resolve_alias_body(&symbol);
                self.in_static_member_type = saved_static;
                self.type_alias_links.get_or_default(&symbol).declared_type =
                    Some(Arc::clone(&found));
                found
            })
        } else {

            let (tp_symbols, type_node) = self.collect_alias_type_params_and_body(&symbol);
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

    pub(crate) fn resolve_alias_body(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        for decl in &symbol.declarations {
            if let NodeData::TypeAliasDeclaration(data) = &decl.data {

                self.push_scope(decl);
                let result = self.get_type_from_type_node(&data.type_node);
                self.pop_scope();
                return result;
            }
        }
        self.error_type()
    }

    pub(crate) fn collect_alias_type_params_and_body(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> (Vec<Arc<Symbol>>, Arc<Node>) {
        let mut tp_symbols = Vec::new();
        let mut type_node = None;
        for decl in &symbol.declarations {
            if let NodeData::TypeAliasDeclaration(data) = &decl.data {
                type_node = Some(Arc::clone(&data.type_node));
                if let Some(tps) = &data.type_parameters {
                    for tp in tps.iter() {
                        if let Some(tp_sym) = self.program.symbol_map().symbol_of(tp) {
                            tp_symbols.push(Arc::clone(tp_sym));
                        }
                    }
                }
                break;
            }
        }
        (
            tp_symbols,
            type_node.unwrap_or_else(|| Arc::clone(&symbol.declarations[0])),
        )
    }

    pub(crate) fn resolve_interface_type(
        &mut self,
        symbol: &Arc<Symbol>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Type> {
        let arg_types = type_arguments.map(|nodes| {
            nodes
                .iter()
                .map(|a| self.get_type_from_type_node(a))
                .collect()
        });
        self.resolve_interface_type_ex(symbol, arg_types)
    }

    pub(crate) fn resolve_interface_type_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        type_args: Option<Vec<Arc<Type>>>,
    ) -> Arc<Type> {

        let has_type_args = type_args.is_some();
        if !has_type_args {
            if let Some(cached) = self
                .type_alias_links
                .get(symbol)
                .and_then(|l| l.declared_type.clone())
            {
                if !crate::checker::utilities::is_type_error(&cached) {
                    return cached;
                }
            }
        }

        let instantiation_key: Option<Vec<usize>> = type_args.as_ref().map(|args| {
            let mut key = Vec::with_capacity(args.len() + 1);
            key.push(Arc::as_ptr(symbol) as *const Symbol as usize);
            key.extend(
                args.iter()
                    .map(|t| Arc::as_ptr(t) as *const crate::checker::types::Type as usize),
            );
            key
        });

        let pinned_args: Option<Vec<Arc<Type>>> = type_args.clone();
        if let Some(key) = &instantiation_key
            && let Some(cached) = self.interface_instantiation_cache.get(key)
        {
            return Arc::clone(&cached.1);
        }

        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {

            self.heritage_degraded_events += 1;
            return self.error_type();
        }

        let interface_decls: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .filter(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)))
            .cloned()
            .collect();

        let epoch_at_entry = self.heritage_degraded_events;
        let mut heritage_degraded = false;
        let result = match interface_decls.first() {
            Some(first) => {
                let data = match &first.data {
                    NodeData::InterfaceDeclaration(d) => d,
                    _ => unreachable!(),
                };

                let tp_symbols = match &data.type_parameters {
                    Some(tps) => {
                        let sym_map = self.program.symbol_map();
                        let collected: Vec<Arc<Symbol>> = tps
                            .iter()
                            .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                            .collect();
                        collected
                    }
                    None => Vec::new(),
                };

                let arg_types: Vec<Arc<Type>> = type_args.unwrap_or_default();
                if has_type_args {
                    let mut mapping = HashMap::new();
                    for (i, tp_sym) in tp_symbols.iter().enumerate() {
                        if let Some(arg) = arg_types.get(i) {
                            let k = Arc::as_ptr(tp_sym) as *const crate::ast::Symbol;
                            mapping.insert(k, Arc::clone(arg));
                        }
                    }
                    for decl in &interface_decls {
                        let NodeData::InterfaceDeclaration(d) = &decl.data else {
                            continue;
                        };
                        let Some(tps) = &d.type_parameters else {
                            continue;
                        };
                        let sym_map = self.program.symbol_map();
                        for (i, tp) in tps.iter().enumerate() {
                            let Some(tp_sym) = sym_map.symbol_of(tp) else {
                                continue;
                            };

                            let idx = if let Some(first_sym) = tp_symbols.get(i) {
                                if first_sym.name == tp_sym.name { i } else {
                                    tp_symbols.iter().position(|s| s.name == tp_sym.name).unwrap_or(i)
                                }
                            } else {
                                i
                            };
                            if let Some(arg) = arg_types.get(idx) {
                                let k = Arc::as_ptr(tp_sym) as *const crate::ast::Symbol;
                                mapping.insert(k, Arc::clone(arg));
                            }
                        }
                    }
                    self.type_argument_stack.push(mapping);
                }

                self.push_scope(
                    symbol
                        .declarations
                        .iter()
                        .next()
                        .expect("interface has a declaration"),
                );

                let merged_members: Vec<Arc<Node>> = interface_decls
                    .iter()
                    .flat_map(|decl| match &decl.data {
                        NodeData::InterfaceDeclaration(d) => d.members.iter().cloned(),
                        _ => unreachable!(),
                    })
                    .collect();
                let merged_list = Arc::new(NodeList::new(merged_members));

                let saved_static = self.in_static_member_type;
                self.in_static_member_type = false;
                let own_result = self.build_interface_type_from_members(&merged_list);
                self.in_static_member_type = saved_static;

                let mut base_types: Vec<(Arc<Node>, Arc<Type>)> = Vec::new();
                for decl in &interface_decls {
                    if let NodeData::InterfaceDeclaration(d) = &decl.data {
                        if let Some(heritage) = &d.heritage_clauses {
                            for clause in heritage.iter() {
                                if let NodeData::HeritageClause(hc) = &clause.data
                                    && hc.token == SyntaxKind::ExtendsKeyword
                                {
                                    for type_ref in hc.types.iter() {
                                        let bt = self.get_type_from_type_node(type_ref);

                                        if crate::checker::utilities::is_type_error(&bt)
                                            && !heritage_degraded
                                        {
                                            heritage_degraded = true;
                                            self.heritage_degraded_events += 1;
                                        }
                                        base_types.push((Arc::clone(type_ref), bt));
                                    }
                                }
                            }
                        }
                    }
                }
                self.pop_scope();
                if has_type_args {
                    self.type_argument_stack.pop();
                }
                let result = if base_types.is_empty() {
                    own_result.clone()
                } else {
                    let mut merged = own_result.clone();
                    for (_, base) in &base_types {
                        merged = self.merge_interface_type_with_base(&merged, base);
                    }
                    merged
                };

                if !has_type_args && !base_types.is_empty() {
                    let own_structured = match &own_result.data {
                        TypeData::Object(o) => Some(&o.structured),
                        _ => None,
                    };
                    let name_loc = interface_decls.first().and_then(|d| {
                        match &d.data {
                            NodeData::InterfaceDeclaration(d) => Some(d.name.loc),
                            _ => None,
                        }
                    });
                                            if let (Some(own), Some(name_loc)) = (own_structured, name_loc) {
                        for (type_ref_node, base) in &base_types {

                            let dedup_key = (
                                Arc::as_ptr(symbol) as *const crate::ast::Symbol,
                                Arc::as_ptr(type_ref_node) as *const crate::ast::Node,
                            );
                            if self.interface_extends_reported.contains(&dedup_key) {
                                continue;
                            }
                            let base_structured = match &base.data {
                                TypeData::Object(o) => Some(&o.structured),
                                _ => None,
                            };
                            let Some(base_structured) = base_structured else {
                                continue;
                            };
                            for own_prop in &own.properties {
                                let Some(base_prop) = base_structured
                                    .members
                                    .get(&own_prop.name)
                                else {
                                    continue;
                                };
                                let derived_type = self
                                    .value_symbol_links
                                    .get(own_prop)
                                    .and_then(|l| l.resolved_type.clone());
                                let base_type = self
                                    .value_symbol_links
                                    .get(base_prop)
                                    .and_then(|l| l.resolved_type.clone());
                                if let (Some(dt), Some(bt)) = (derived_type, base_type) {

                                    let bt = match bt.symbol.as_ref() {
                                        Some(bsym) => {
                                            let tps = self.declared_type_parameter_types(bsym);
                                            if !tps.is_empty()
                                                && bt.as_object().is_none_or(|o| {
                                                    o.type_arguments.is_empty()
                                                })
                                            {
                                                let anys: Vec<Arc<Type>> = std::iter::repeat(
                                                    self.get_any_type(),
                                                )
                                                .take(tps.len())
                                                .collect();
                                                self.resolve_interface_type_ex(
                                                    bsym,
                                                    Some(anys),
                                                )
                                            } else {
                                                bt
                                            }
                                        }
                                        None => bt,
                                    };

                                    let saved_chain =
                                        std::mem::take(&mut self.relater_error_chain);
                                    let was_active = self.relater_chain_active;
                                    self.relater_chain_active = true;
                                    let incompatible = !self.is_type_assignable_to(&dt, &bt);
                                    let captured = std::mem::replace(
                                        &mut self.relater_error_chain,
                                        saved_chain,
                                    );
                                    self.relater_chain_active = was_active;
                                    if incompatible {
                                        self.interface_extends_reported.insert(dedup_key);
                                        let base_name = base
                                            .symbol
                                            .as_ref()
                                            .map(|s| s.name.clone())
                                            .unwrap_or_default();
                                        let file = self.current_file.clone();
                                        let mut diag = crate::ast::Diagnostic::new(
                                            file,
                                            name_loc,
                                            crate::diagnostics::messages_generated::
                                                INTERFACE_0_INCORRECTLY_EXTENDS_INTERFACE_1,
                                            vec![symbol.name.clone(), base_name],
                                        );

                                        let dt_str = self.type_to_string(&dt);
                                        let bt_str = self.type_to_string(&bt);
                                        self.relater_error_chain = captured;
                                        self.relater_chain_active = true;
                                        self.push_relation_head_with_tp_note(
                                            &dt,
                                            &bt,
                                            crate::diagnostics::messages_generated::
                                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                            vec![dt_str, bt_str],
                                        );
                                        self.relater_report_error(
                                            crate::diagnostics::messages_generated::
                                                TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                                            vec![self.chain_property_arg_name(own_prop)],
                                        );
                                        let entries = std::mem::take(
                                            &mut self.relater_error_chain,
                                        );
                                        self.relater_chain_active = was_active;

                                        let mut child: Option<
                                            crate::ast::Diagnostic,
                                        > = None;
                                        for entry in entries
                                            .iter()
                                            .filter(|e| {
                                                !e.message.elided_in_compatibility_pyramid
                                            })
                                        {
                                            let mut d = crate::ast::Diagnostic::new(
                                                None,
                                                name_loc,
                                                entry.message,
                                                entry.args.clone(),
                                            );
                                            if let Some(c) = child.take() {
                                                d.message_chain = vec![c];
                                            }
                                            child = Some(d);
                                        }
                                        if let Some(c) = child {
                                            diag.message_chain = vec![c];
                                        }
                                        self.diagnostics.add(diag);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                {
                    let result_mut = Arc::as_ptr(&result) as *mut crate::checker::types::Type;
                    unsafe {
                        (*result_mut).symbol = Some(Arc::clone(symbol));
                        if has_type_args
                            && let TypeData::Object(o) = &mut (*result_mut).data
                        {
                            o.type_arguments = arg_types.clone();
                        }
                    }
                }
                result
            }
            None => self.error_type(),
        };
        self.pop_type_resolution();

        if self.heritage_degraded_events != epoch_at_entry {
            heritage_degraded = true;
        }

        let mut degraded_accepted = false;
        if heritage_degraded {
            let sym_key = Arc::as_ptr(symbol) as *const crate::ast::Symbol as usize;
            let retries = self.heritage_retry_counts.entry(sym_key).or_insert(0);
            *retries += 1;
            degraded_accepted = *retries > crate::checker::checker::HERITAGE_RETRY_LIMIT;
        }
        let cache_result = !heritage_degraded || degraded_accepted;
        if degraded_accepted && self.heritage_degraded_events != epoch_at_entry {
            self.heritage_degraded_events = epoch_at_entry;
        }
        if heritage_degraded {

            self.degraded_type_ptrs.insert(result.id);
        }
        if !has_type_args && cache_result {
            self.type_alias_links.get_or_default(symbol).declared_type = Some(result.clone());
        }
        if let Some(key) = instantiation_key {
            if cache_result {

                let pin = pinned_args.clone().unwrap_or_default();
                self.interface_instantiation_cache
                    .insert(key, (pin, Arc::clone(&result)));
            }
        }
        result
    }

    pub(crate) fn build_interface_type_from_members(
        &mut self,
        members: &Arc<NodeList>,
    ) -> Arc<Type> {
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        let mut index_infos: Vec<Arc<crate::checker::IndexInfo>> = Vec::new();

        let mut call_signatures: Vec<Arc<Signature>> = Vec::new();
        let mut construct_signatures: Vec<Arc<Signature>> = Vec::new();
        for member in members.iter() {
            match &member.data {
                NodeData::PropertySignatureDeclaration(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }
                    let mut prop_type = self.get_type_from_type_node(&data.type_node);
                    let is_optional = data
                        .postfix_token
                        .as_ref()
                        .map(|t| t.kind == SyntaxKind::QuestionToken)
                        .unwrap_or(false);

                    if is_optional {
                        prop_type = self.get_optional_type(prop_type);
                    }
                    let mut flags = SymbolFlags::Property;
                    if is_optional {
                        flags |= SymbolFlags::Optional;
                    }
                    let mut symbol = Symbol::new(flags, name.clone());

                    symbol.declarations = vec![Arc::clone(member)];

                    if let Some(m) = &data.modifiers {
                        if m.modifier_flags.contains(ModifierFlags::Readonly) {
                            symbol.check_flags |= CheckFlags::Readonly;
                        }
                    }
                    let symbol = Arc::new(symbol);
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(prop_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::MethodSignatureDeclaration(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }

                    self.push_scope(member);
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                         false,
                         None,
                         Some(Arc::clone(member)),
                    );
                    self.pop_scope();

                    if let Some(existing) = symbol_table.get(&name).cloned() {
                        let existing_type = self
                            .value_symbol_links
                            .get(&existing)
                            .and_then(|l| l.resolved_type.clone());
                        let merged_sigs = existing_type
                            .as_ref()
                            .and_then(|t| t.as_structured().map(|s| s.call_signatures().to_vec()))
                            .unwrap_or_default();
                        let mut all_sigs = merged_sigs;
                        all_sigs.push(sig);
                        let fn_type = self.create_function_or_constructor_type(all_sigs, false);
                        self.value_symbol_links.insert(
                            &existing,
                            ValueSymbolLinks {
                                resolved_type: Some(fn_type),
                                ..Default::default()
                            },
                        );
                        continue;
                    }
                    let fn_type = self.create_function_or_constructor_type(vec![sig], false);
                    let symbol = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(fn_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::IndexSignatureDeclaration(data) => {
                    let mut key_type = None;
                    let value_type;
                    if let Some(param) = data.parameters.iter().next() {
                        if let NodeData::ParameterDeclaration(pd) = &param.data {
                            key_type = pd
                                .type_node
                                .as_ref()
                                .map(|t| self.get_type_from_type_node(t));
                        }
                    }
                    value_type = Some(self.get_type_from_type_node(&data.type_node));
                    let is_readonly = member
                        .modifiers()
                        .as_ref()
                        .is_some_and(|m| m.flags().contains(ModifierFlags::Readonly));
                    index_infos.push(Arc::new(crate::checker::IndexInfo {
                        key_type,
                        value_type,
                        is_readonly,
                        declaration: Some(Arc::clone(member)),
                        index_symbol: None,
                        components: Vec::new(),
                    }));
                }

                NodeData::PropertyDeclaration(data) => {
                    if is_static_modifier(&data.modifiers) {
                        continue;
                    }
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }
                    let mut prop_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => match data.initializer.as_ref() {
                            Some(init) => {

                                let raw = match &init.data {
                                    NodeData::Identifier(_) => {
                                        match self.resolve_identifier(init) {
                                            Some(sym) if sym.flags.intersects(
                                                SymbolFlags::BlockScopedVariable
                                                    | SymbolFlags::FunctionScopedVariable,
                                            ) => self.get_type_of_symbol(&sym),
                                            _ => self.get_type_of_node(init),
                                        }
                                    }
                                    _ => self.get_type_of_node(init),
                                };
                                let is_readonly = data
                                    .modifiers
                                    .as_ref()
                                    .is_some_and(|m| {
                                        m.modifier_flags.contains(ModifierFlags::Readonly)
                                    });
                                let widened = if is_readonly {
                                    raw
                                } else if self.is_empty_array_literal(init) {

                                    if self.strict_null_checks {
                                        self.get_widened_literal_type(&raw)
                                    } else {
                                        self.create_array_type(self.get_any_type())
                                    }
                                } else {
                                    self.get_widened_literal_type(&raw)
                                };
                                let regularized =
                                    self.get_regular_type_of_literal_type(&widened);
                                self.widen_initializer_type(&regularized)
                            }
                            None => self.get_any_type(),
                        },
                    };
                    let is_optional = data
                        .postfix_token
                        .as_ref()
                        .map(|t| t.kind == SyntaxKind::QuestionToken)
                        .unwrap_or(false);
                    if is_optional {
                        prop_type = self.get_optional_type(prop_type);
                    }
                    let mut flags = SymbolFlags::Property;
                    if is_optional {
                        flags |= SymbolFlags::Optional;
                    }
                    let mut symbol = Symbol::new(flags, name.clone());

                    symbol.declarations.push(Arc::clone(member));

                    if let Some(m) = &data.modifiers {
                        if m.modifier_flags.contains(ModifierFlags::Readonly) {
                            symbol.check_flags |= CheckFlags::Readonly;
                        }
                    }
                    let symbol = Arc::new(symbol);
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(prop_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::MethodDeclaration(data) => {
                    if is_static_modifier(&data.modifiers) {
                        continue;
                    }
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }

                    self.push_scope(member);
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                         false,
                         None,
                         Some(Arc::clone(member)),
                    );
                    self.pop_scope();

                    if let Some(existing) = symbol_table.get(&name).cloned() {
                        if data.body.is_some() {

                            let existing_mut =
                                Arc::as_ptr(&existing) as *mut Symbol;
                            unsafe {
                                (*existing_mut).declarations.push(Arc::clone(member));
                            }
                            continue;
                        }
                        let existing_type = self
                            .value_symbol_links
                            .get(&existing)
                            .and_then(|l| l.resolved_type.clone());
                        let merged_sigs = existing_type
                            .as_ref()
                            .and_then(|t| {
                                t.as_structured().map(|s| s.call_signatures().to_vec())
                            })
                            .unwrap_or_default();
                        let mut all_sigs = merged_sigs;
                        all_sigs.push(sig);
                        let fn_type =
                            self.create_function_or_constructor_type(all_sigs, false);
                        self.value_symbol_links.insert(
                            &existing,
                            ValueSymbolLinks {
                                resolved_type: Some(fn_type),
                                ..Default::default()
                            },
                        );
                        continue;
                    }
                    let fn_type = self.create_function_or_constructor_type(vec![sig], false);
                    let mut symbol = Symbol::new(SymbolFlags::Property, name.clone());

                    symbol.declarations.push(Arc::clone(member));
                    let symbol = Arc::new(symbol);
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(fn_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::GetAccessorDeclaration(data) => {
                    if is_static_modifier(&data.modifiers) {
                        continue;
                    }
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }

                    let prop_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    match symbol_table.get(&name).cloned() {
                        Some(existing) => {

                            let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                            unsafe {
                                (*existing_mut).flags |= SymbolFlags::GetAccessor;
                                (*existing_mut).declarations.push(Arc::clone(member));
                            }
                            self.value_symbol_links.insert(
                                &existing,
                                ValueSymbolLinks {
                                    resolved_type: Some(prop_type),
                                    ..Default::default()
                                },
                            );
                        }
                        None => {
                            let mut symbol =
                                Symbol::new(SymbolFlags::Property | SymbolFlags::GetAccessor, name.clone());
                            symbol.declarations.push(Arc::clone(member));
                            let symbol = Arc::new(symbol);
                            self.value_symbol_links.insert(
                                &symbol,
                                ValueSymbolLinks {
                                    resolved_type: Some(prop_type),
                                    ..Default::default()
                                },
                            );
                            symbol_table.insert(name, Arc::clone(&symbol));
                            props.push(symbol);
                        }
                    }
                }
                NodeData::SetAccessorDeclaration(data) => {
                    if is_static_modifier(&data.modifiers) {
                        continue;
                    }
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }

                    let prop_type = data
                        .parameters
                        .iter()
                        .next()
                        .and_then(|p| {
                            if let NodeData::ParameterDeclaration(pd) = &p.data {
                                pd.type_node.as_ref().map(|tn| self.get_type_from_type_node(tn))
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| self.get_any_type());
                    match symbol_table.get(&name).cloned() {
                        Some(existing) => {

                            let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                            unsafe {
                                (*existing_mut).flags |= SymbolFlags::SetAccessor;
                                (*existing_mut).declarations.push(Arc::clone(member));
                            }
                        }
                        None => {
                            let mut symbol =
                                Symbol::new(SymbolFlags::Property | SymbolFlags::SetAccessor, name.clone());
                            symbol.declarations.push(Arc::clone(member));
                            let symbol = Arc::new(symbol);
                            self.value_symbol_links.insert(
                                &symbol,
                                ValueSymbolLinks {
                                    resolved_type: Some(prop_type),
                                    ..Default::default()
                                },
                            );
                            symbol_table.insert(name, Arc::clone(&symbol));
                            props.push(symbol);
                        }
                    }
                }
                NodeData::CallSignatureDeclaration(data) => {

                    let suppress = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.file_name.starts_with("bundled://"));
                    if suppress {
                        self.push_ts2304_suppression();
                    }

                    self.push_scope(member);
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                         false,
                         None,
                         Some(Arc::clone(member)),
                    );
                    self.pop_scope();
                    if suppress {
                        self.pop_ts2304_suppression();
                    }
                    call_signatures.push(sig);
                }
                NodeData::ConstructSignatureDeclaration(data) => {
                    let suppress = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.file_name.starts_with("bundled://"));
                    if suppress {
                        self.push_ts2304_suppression();
                    }

                    self.push_scope(member);
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                         true,
                         None,
                         Some(Arc::clone(member)),
                    );
                    self.pop_scope();
                    if suppress {
                        self.pop_ts2304_suppression();
                    }
                    construct_signatures.push(sig);
                }
                NodeData::ConstructorDeclaration(data) => {

                    for param in data.parameters.iter() {
                        let NodeData::ParameterDeclaration(pd) = &param.data else {
                            continue;
                        };
                        if pd.name.kind != SyntaxKind::Identifier {
                            continue;
                        }
                        let Some(modifiers) = &pd.modifiers else {
                            continue;
                        };
                        if !modifiers.modifier_flags.intersects(
                            ModifierFlags::Public
                                | ModifierFlags::Private
                                | ModifierFlags::Protected
                                | ModifierFlags::Readonly,
                        ) {
                            continue;
                        }
                        let name = pd.name.text().to_string();
                        if name.is_empty() || symbol_table.get(&name).is_some() {
                            continue;
                        }
                        let prop_type = match pd.type_node.as_ref() {
                            Some(tn) => self.get_type_from_type_node(tn),
                            None => match pd.initializer.as_ref() {
                                Some(init) => self.get_type_of_node(init),
                                None => self.get_any_type(),
                            },
                        };
                        let mut symbol = Symbol::new(SymbolFlags::Property, name.clone());

                        symbol.declarations.push(Arc::clone(param));
                        if modifiers.modifier_flags.contains(ModifierFlags::Readonly) {
                            symbol.check_flags |= CheckFlags::Readonly;
                        }
                        let symbol = Arc::new(symbol);
                        self.value_symbol_links.insert(
                            &symbol,
                            ValueSymbolLinks {
                                resolved_type: Some(prop_type),
                                ..Default::default()
                            },
                        );
                        symbol_table.insert(name, Arc::clone(&symbol));
                        props.push(symbol);
                    }
                }
                _ => {}
            }
        }

        let call_signature_count = call_signatures.len();
        let mut signatures = call_signatures;
        signatures.extend(construct_signatures);
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    signatures,
                    call_signature_count,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn merge_interface_type_with_base(
        &mut self,
        derived: &Arc<Type>,
        base: &Arc<Type>,
    ) -> Arc<Type> {
        if base.flags.contains(TypeFlags::Any) {
            return Arc::clone(derived);
        }
        let derived_data = match &derived.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };
        let base_data = match &base.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();

        for prop in &derived_data.properties {
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }
        for prop in &base_data.properties {
            if symbol_table.get(&prop.name).is_some() {
                continue;
            }
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }
        let mut index_infos = derived_data.index_infos.clone();
        index_infos.extend(base_data.index_infos.iter().cloned());

        let mut call_signatures: Vec<Arc<Signature>> =
            derived_data.call_signatures().to_vec();
        let derived_call_count = call_signatures.len();
        call_signatures.extend(base_data.call_signatures().iter().cloned());
        let mut signatures = call_signatures;
        signatures.extend(derived_data.construct_signatures().iter().cloned());
        signatures.extend(base_data.construct_signatures().iter().cloned());
        let merged = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    signatures,
                    call_signature_count: derived_call_count
                        + base_data.call_signatures().len(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        });

        let merged_degraded = self.degraded_type_ptrs.contains(&base.id)
            || self.degraded_type_ptrs.contains(&derived.id);
        if merged_degraded {
            self.degraded_type_ptrs.insert(merged.id);
        }
        merged
    }

    pub(crate) fn resolve_enum_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {

        if let Some(cached) = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }

        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            return self.error_type();
        }

        let sym_map = self.program.symbol_map();
        let mut entries: Vec<(Option<Arc<Symbol>>, String, Option<Arc<Node>>)> = Vec::new();
        for decl in symbol.declarations.iter() {
            if let NodeData::EnumDeclaration(data) = &decl.data {
                for member_node in data.members.iter() {
                    let NodeData::EnumMember(member) = &member_node.data else {
                        continue;
                    };
                    let member_name = member.name.text().to_string();
                    let member_sym = sym_map.symbol_of(member_node).map(Arc::clone);
                    entries.push((member_sym, member_name, member.initializer.clone()));
                }
            }
        }
        let result = if entries.is_empty() {
            self.error_type()
        } else {
            let mut member_types: Vec<Arc<Type>> = Vec::new();
            let mut next_value: Option<f64> = Some(0.0);
            for (member_sym, member_name, initializer) in &entries {
                let base = match initializer {
                    Some(init) => {

                        let t = self.get_type_of_node(init);

                        if t.flags.contains(TypeFlags::NumberLiteral) {
                            if let TypeData::Literal(LiteralTypeData {
                                value: LiteralValue::Number(n),
                                ..
                            }) = &t.data
                            {
                                next_value = Some(n.0 + 1.0);
                            }
                        } else if t.flags.contains(TypeFlags::StringLiteral) {

                            next_value = None;
                        }
                        t
                    }
                    None => match next_value {
                        Some(v) => {
                            next_value = Some(v + 1.0);
                            self.get_number_literal_type(jsnum::Number::from(v))
                        }
                        None => {

                            self.get_any_type()
                        }
                    },
                };

                let member_type = if base
                    .flags
                    .intersects(TypeFlags::NumberLiteral | TypeFlags::StringLiteral)
                {
                    let value = match &base.data {
                        TypeData::Literal(lit) => lit.value.clone(),
                        _ => LiteralValue::None,
                    };
                    let enum_literal_flags = base.flags | TypeFlags::EnumLiteral;
                    let mut regular_ty = Type::new(
                        enum_literal_flags,
                        TypeData::Literal(LiteralTypeData {
                            value: value.clone(),
                            fresh_type: OnceLock::new(),
                            regular_type: OnceLock::new(),
                        }),
                    );
                    regular_ty.symbol = member_sym.clone();
                    let regular_ty = Arc::new(regular_ty);
                    let mut fresh_ty = Type::new(
                        enum_literal_flags,
                        TypeData::Literal(LiteralTypeData {
                            value,
                            fresh_type: OnceLock::new(),
                            regular_type: OnceLock::from(Arc::clone(&regular_ty)),
                        }),
                    );
                    fresh_ty.symbol = member_sym.clone();
                    let fresh_ty = Arc::new(fresh_ty);

                    if let TypeData::Literal(reg_lit) = &regular_ty.data {
                        let _ = reg_lit.fresh_type.set(Arc::clone(&fresh_ty));
                    }
                    if let Some(ms) = member_sym {
                        self.value_symbol_links.insert(
                            ms,
                            ValueSymbolLinks {
                                resolved_type: Some(fresh_ty),
                                ..Default::default()
                            },
                        );
                    }
                    regular_ty
                } else {

                    if let Some(ms) = member_sym {
                        self.value_symbol_links.insert(
                            ms,
                            ValueSymbolLinks {
                                resolved_type: Some(Arc::clone(&base)),
                                ..Default::default()
                            },
                        );
                    }
                    base
                };
                let _ = member_name;
                member_types.push(member_type);
            }
            match member_types.len() {
                0 => self.never_type(),
                1 => member_types.into_iter().next().unwrap(),
                _ => self.get_union_type(member_types),
            }
        };
        self.pop_type_resolution();
        self.type_alias_links.get_or_default(symbol).declared_type = Some(result.clone());
        result
    }

    pub(crate) fn get_type_of_prototype_property(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        let Some(parent) = symbol.parent.clone() else {
            return self.get_any_type();
        };
        let Some(class_decl) = parent
            .declarations
            .iter()
            .find(|d| matches!(d.data, NodeData::ClassDeclaration(_)))
            .cloned()
        else {
            return self.get_any_type();
        };
        let ctor_type = self.get_type_of_class_declaration(&class_decl);
        let instance_type = ctor_type
            .as_structured()
            .and_then(|s| s.construct_signatures().first().cloned())
            .and_then(|sig| self.get_return_type_of_signature(&sig))
            .unwrap_or_else(|| self.get_any_type());
        let tp_types: Vec<Arc<Type>> = match &class_decl.data {
            NodeData::ClassDeclaration(d) => match &d.type_parameters {
                Some(tps) => {
                    let sym_map = self.program.symbol_map();
                    let tp_syms: Vec<Arc<Symbol>> = tps
                        .iter()
                        .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                        .collect();
                    tp_syms
                        .iter()
                        .map(|s| self.get_type_parameter_from_symbol(s))
                        .collect()
                }
                None => Vec::new(),
            },
            _ => Vec::new(),
        };
        if tp_types.is_empty() {
            return instance_type;
        }
        let any_t = self.get_any_type();
        let anys: Vec<Arc<Type>> = tp_types.iter().map(|_| Arc::clone(&any_t)).collect();
        self.substitute_infer_type_parameters(&instance_type, &tp_types, &anys)
    }

    pub(crate) fn get_type_parameter_from_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {

        if let Some(links) = self.type_alias_links.get(symbol) {
            if let Some(ref t) = links.declared_type {
                return Arc::clone(t);
            }
        }
        let sym_key = Arc::as_ptr(symbol) as usize;
        if !self.type_parameter_resolving.insert(sym_key) {

            return Arc::new(Type {
                flags: TypeFlags::TypeParameter,
                object_flags: ObjectFlags::None,
                id: crate::checker::types::next_type_id(),
                symbol: Some(Arc::clone(symbol)),
                alias: None,
                data: TypeData::TypeParameter(TypeParameterData {
                    constrained: ConstrainedTypeData::default(),
                    constraint: None,
                    target: None,
                    mapper: None,
                    is_this_type: false,
                    resolved_default_type: OnceLock::new(),
                }),
            });
        }
        let mut constraint: Option<Arc<Type>> = None;
        for decl in &symbol.declarations {
            if let NodeData::TypeParameterDeclaration(data) = &decl.data {
                if let Some(constraint_node) = &data.constraint {
                    constraint = Some(self.get_type_from_type_node(constraint_node));
                }
                break;
            }
        }

        if let Some(c) = &constraint
            && self.constraint_chain_is_circular(sym_key, c)
        {
            if self.ts2313_reported.insert(sym_key) {
                let loc = symbol
                    .declarations
                    .iter()
                    .find_map(|d| match &d.data {
                        NodeData::TypeParameterDeclaration(td) => {
                            td.constraint.as_ref().map(|cn| cn.loc)
                        }
                        _ => None,
                    })
                    .or_else(|| symbol.declarations.first().map(|d| d.loc))
                    .unwrap_or_default();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    loc,
                    crate::diagnostics::messages_generated::
                        TYPE_PARAMETER_0_HAS_A_CIRCULAR_CONSTRAINT,
                    vec![symbol.name.clone()],
                ));
            }
            constraint = None;
        }
        let tp = Arc::new(Type {
            flags: TypeFlags::TypeParameter,
            object_flags: ObjectFlags::None,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::TypeParameter(TypeParameterData {
                constrained: ConstrainedTypeData::default(),
                constraint,
                target: None,
                mapper: None,
                is_this_type: false,
                resolved_default_type: OnceLock::new(),
            }),
        });
        self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&tp));
        self.type_parameter_resolving.remove(&sym_key);
        tp
    }

    pub(crate) fn constraint_chain_is_circular(&self, start_key: usize, constraint: &Arc<Type>) -> bool {
        let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut current = constraint;
        for _ in 0..50 {
            let TypeData::TypeParameter(tp) = &current.data else {
                return false;
            };
            let Some(sym) = &current.symbol else { return false };
            let key = Arc::as_ptr(sym) as usize;
            if !visited.insert(key) {
                return true;
            }
            match &tp.constraint {
                Some(next) => current = next,
                None => return key == start_key,
            }
        }
        false
    }

    pub(crate) fn resolve_namespace_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {

        if let Some(cached) = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }

        let mut members: Vec<(String, Arc<Symbol>)> = symbol
            .exports
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();

        if self.ambient_namespace_locals_visible(symbol) {
            let local_members: Vec<(String, Arc<Symbol>)> = symbol
                .declarations
                .iter()
                .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                .filter_map(|d| {
                    self.program
                        .symbol_map()
                        .locals
                        .get(&d.id())
                        .map(|l| {
                            l.iter()
                                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                                .collect::<Vec<(String, Arc<Symbol>)>>()
                        })
                })
                .flatten()
                .collect();
            for (k, v) in local_members {
                if !members.iter().any(|(mk, _)| *mk == k) {
                    members.push((k, v));
                }
            }
        }

        let mut file_exported: Vec<(String, Arc<Symbol>)> = Vec::new();
        if symbol
            .declarations
            .iter()
            .any(|d| d.kind == SyntaxKind::SourceFile)
            && members.is_empty()
        {
            let sym_map = self.program.symbol_map();

            let mut wanted: Vec<(String, Option<String>)> = Vec::new();
            let mut default_node: Option<Arc<Node>> = None;
            self.for_each_module_statement(symbol, |stmt| {
                let has_export =
                    stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export);
                match &stmt.data {
                    NodeData::ExportDeclaration(d) => {
                        if let Some(clause) = &d.export_clause
                            && let NodeData::NamedExports(ne) = &clause.data
                        {
                            for el in ne.elements.iter() {
                                if let NodeData::ExportSpecifier(spec) = &el.data {
                                    let exported = spec
                                        .name
                                        .text()
                                        .trim_matches(['"', '\'', '`'])
                                        .to_string();
                                    let local = spec
                                        .property_name
                                        .as_ref()
                                        .unwrap_or(&spec.name)
                                        .text()
                                        .trim_matches(['"', '\'', '`'])
                                        .to_string();
                                    wanted.push((exported, Some(local)));
                                }
                            }
                        }
                    }
                    NodeData::ExportAssignment(ea) => {
                        if !ea.is_export_equals && default_node.is_none() {
                            default_node = Some(Arc::clone(stmt));
                        }
                    }
                    NodeData::VariableStatement(vs) if has_export => {
                        if let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data {
                            for decl in vdl.declarations.iter() {
                                if let Some(name) = decl.name() {
                                    wanted.push((name.text().to_string(), None));
                                }
                            }
                        }
                    }
                    _ if has_export => {
                        if let Some(name) = stmt.name() {
                            wanted.push((name.text().to_string(), None));
                        }
                    }
                    _ => {}
                }
                false
            });
            let file_node = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::SourceFile);
            let locals = file_node.and_then(|f| sym_map.locals.get(&f.id()));
            for (exported, clause_local) in wanted.iter() {
                if members.iter().any(|(k, _)| *k == *exported)
                    || file_exported.iter().any(|(k, _)| *k == *exported)
                {
                    continue;
                }
                let lookup = clause_local.as_deref().unwrap_or(exported);
                if let Some(s) = locals.and_then(|l| l.get(lookup).cloned()) {
                    file_exported.push((exported.clone(), s));
                } else if let Some(s) = symbol.members.get(lookup).cloned() {
                    file_exported.push((exported.clone(), s));
                }
            }
            if let Some(node) = default_node
                && !members.iter().any(|(k, _)| k == "default")
                && !file_exported.iter().any(|(k, _)| k == "default")
            {

                let s = sym_map
                    .symbol_of(&node)
                    .cloned()
                    .or_else(|| node.expression().and_then(|e| sym_map.symbol_of(e).cloned()));
                if let Some(s) = s {
                    file_exported.push(("default".to_string(), s));
                }
            }
        }

        let mut reexported: Vec<(String, Arc<Symbol>)> = Vec::new();
        {
            let mut clause_specs: Vec<(String, String, String)> = Vec::new();
            self.for_each_module_statement(symbol, |stmt| {
                if let NodeData::ExportDeclaration(d) = &stmt.data
                    && let Some(clause) = &d.export_clause
                    && let NodeData::NamedExports(ne) = &clause.data
                    && let Some(module_spec) = &d.module_specifier
                {
                    for el in ne.elements.iter() {
                        if let NodeData::ExportSpecifier(spec) = &el.data {
                            let exported = spec
                                .name
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            let imported = spec
                                .property_name
                                .as_ref()
                                .unwrap_or(&spec.name)
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            let module_text = module_spec
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            if !exported.is_empty() && !module_text.is_empty() {
                                clause_specs.push((exported, imported, module_text));
                            }
                        }
                    }
                }
                false
            });
            for (exported, imported, module_text) in clause_specs {
                if members.iter().any(|(k, _)| *k == exported)
                    || reexported.iter().any(|(k, _)| *k == exported)
                {
                    continue;
                }
                let target = self
                    .resolve_module_spec_from(symbol, &module_text)
                    .and_then(|m| self.resolve_module_member_symbol(&m, &imported, 8));
                if let Some(t) = target {
                    reexported.push((exported, t));
                }
            }
        }
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for (name, member_sym) in members.iter().chain(file_exported.iter()).chain(reexported.iter()) {

            if name.starts_with(crate::ast::INTERNAL_SYMBOL_NAME_PREFIX)
                || name == crate::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
            {
                continue;
            }
            let member_type = self.get_type_of_symbol(member_sym);

            let prop_sym = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
            self.value_symbol_links.insert(
                &prop_sym,
                ValueSymbolLinks {
                    resolved_type: Some(member_type),
                    ..Default::default()
                },
            );
            symbol_table.insert(name.clone(), Arc::clone(&prop_sym));
            props.push(prop_sym);
        }
        let result = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        });
        self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&result));
        result
    }
}
