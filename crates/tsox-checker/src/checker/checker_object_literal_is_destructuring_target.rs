#![allow(unused_imports)]

use crate::checker::checker::*;

pub(crate) fn is_assignment_target(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_assignment_target");
    crate::checker::utilities_get_assignment_target::get_assignment_target(node).is_some()
}

pub(crate) fn is_let_or_const_declaration(declaration: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_let_or_const_declaration"); 
    if let Some(parent) = declaration.parent().as_ref() {
        if parent.kind == SyntaxKind::VariableDeclarationList {
            return parent.flags.intersects(NodeFlags::Let | NodeFlags::Const);
        }
    }

    true
}

pub(crate) fn type_contains_undefined(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("type_contains_undefined"); 
    if t.flags.contains(TypeFlags::Undefined) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(type_contains_undefined);
        }
    }
    false
}

pub(crate) fn type_is_possibly_undefined(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("type_is_possibly_undefined"); 
    if t.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(|ct| ct.flags.intersects(TypeFlags::Undefined | TypeFlags::Null));
        }
    }
    false
}

pub(crate) fn type_includes_undefined_only(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("type_includes_undefined_only"); 
    if t.flags.contains(TypeFlags::Undefined) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(|ct| ct.flags.contains(TypeFlags::Undefined));
        }
    }
    false
}

pub(crate) fn type_includes_null_only(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("type_includes_null_only"); 
    if t.flags.contains(TypeFlags::Null) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(|ct| ct.flags.contains(TypeFlags::Null));
        }
    }
    false
}

pub(crate) fn is_entity_name_expression(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_entity_name_expression"); 
    match node.kind {
        SyntaxKind::Identifier => true,
        SyntaxKind::PropertyAccessExpression => match &node.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(data) => {
                is_entity_name_expression(&data.expression)
            }
            _ => false,
        },
        _ => false,
    }
}

pub(crate) fn flatten_union_leaves<'a>(t: &'a Arc<Type>, leaves: &mut Vec<&'a Arc<Type>>) { ::tsox_core::fntrace::enter("flatten_union_leaves"); 
    match t.as_union_or_intersection() {
        Some(u) => {
            for m in &u.types {
                flatten_union_leaves(m, leaves);
            }
        }
        None => leaves.push(t),
    }
}

pub(crate) fn class_declaration_name(class: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("class_declaration_name"); 
    if let tsox_frontend::ast::NodeData::ClassDeclaration(d) = &class.data {
        return d.name.as_ref().map(|n| n.text().to_string());
    }
    None
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ModuleMemberLookup {
    Found,

    LocalNotExported,
    Missing,
}

#[allow(dead_code)]
pub(crate) fn body_assigns_this_property(n: &Arc<Node>, name: &str) -> bool { ::tsox_core::fntrace::enter("body_assigns_this_property"); 
    match &n.data {
        tsox_frontend::ast::NodeData::BinaryExpression(b)
            if b.operator_token.kind == SyntaxKind::EqualsToken =>
        {
            if let tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) = &b.left.data
                && pa.expression.kind == SyntaxKind::ThisKeyword
                && pa.name.kind == SyntaxKind::Identifier
                && pa.name.text() == name
            {
                return true;
            }
        }

        tsox_frontend::ast::NodeData::FunctionDeclaration(_)
        | tsox_frontend::ast::NodeData::FunctionExpression(_)
        | tsox_frontend::ast::NodeData::ArrowFunction(_) => return false,
        _ => {}
    }
    let mut found = false;
    tsox_frontend::ast::node_data_generated::for_each_child(n, |child| {
        if body_assigns_this_property(child, name) {
            found = true;
            true
        } else {
            false
        }
    });
    found
}

impl std::fmt::Debug for Checker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { ::tsox_core::fntrace::enter("fmt"); 
        f.debug_struct("Checker")
            .field("id", &self.id)
            .field("type_count", &self.type_count)
            .field("symbol_count", &self.symbol_count)
            .field("files", &self.files.len())
            .finish()
    }
}

impl Checker {
    pub(crate) fn attach_explicit_type_arguments_cached(
        &mut self,
        t: &Arc<Type>,
        args: Vec<Arc<Type>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("attach_explicit_type_arguments_cached"); 
        let mut key = Vec::with_capacity(args.len() + 1);
        key.push(t.id as usize);
        key.extend(args.iter().map(|a| a.id as usize));
        if let Some(cached) = self.attached_type_args_cache.get(&key) {
            return Arc::clone(&cached.2);
        }
        let rebuilt = attach_explicit_type_arguments(t, args.clone());
        let rebuilt = self.instantiate_attached_reference_members(rebuilt, t, &args);
        // 类成员填充窗口内的 attach 源是半成品（成员解析中途的重入约束
        // 解析），快照驻留会把残缺成员钉死，不缓存待后续完整重算
        if self.filling_class_members.is_empty() {
            self.attached_type_args_cache
                .insert(key, (Arc::clone(t), args, Arc::clone(&rebuilt)));
        }
        rebuilt
    }

    fn instantiate_attached_reference_members(
        &mut self,
        rebuilt: Arc<Type>,
        t: &Arc<Type>,
        args: &[Arc<Type>],
    ) -> Arc<Type> {
        let Some(obj) = t.as_object() else {
            return rebuilt;
        };
        if !obj.type_arguments.is_empty() {
            return rebuilt;
        }
        let Some(sym) = t.symbol.clone() else {
            return rebuilt;
        };
        let params = self.declared_type_parameter_types(&sym);
        if params.is_empty() || params.len() != args.len() {
            return rebuilt;
        }
        if params.iter().zip(args.iter()).all(|(p, a)| Arc::ptr_eq(p, a)) {
            return rebuilt;
        }
        let members = obj.structured.members.clone();
        let properties = obj.structured.properties.clone();
        let mut new_members = members.clone();
        let mut changed = false;
        for (name, member) in members.iter() {
            let Some(old_t) = self.instantiated_member_source_type(member) else {
                continue;
            };
            let new_t = self.substitute_infer_type_parameters(&old_t, &params, args);
            if Arc::ptr_eq(&new_t, &old_t) {
                continue;
            }
            let mut cloned = Symbol::new(member.flags | SymbolFlags::Transient, member.name.clone());
            cloned.check_flags = CheckFlags::Instantiated | member.check_flags;
            cloned.declarations = member.declarations.clone();
            cloned.value_declaration = member.value_declaration.clone();
            if let Some(parent) = member.parent() {
                cloned.set_parent(&parent);
            }
            let cloned = Arc::new(cloned);
            self.value_symbol_links.insert(
                &cloned,
                ValueSymbolLinks {
                    resolved_type: Some(new_t),
                    target: Some(Arc::clone(member)),
                    ..Default::default()
                },
            );
            new_members.insert(name.clone(), cloned);
            changed = true;
        }
        if !changed {
            return rebuilt;
        }
        let new_properties: Vec<Arc<Symbol>> = properties
            .iter()
            .map(|p| {
                new_members
                    .get(&p.name)
                    .cloned()
                    .unwrap_or_else(|| Arc::clone(p))
            })
            .collect();
        let mut rebuilt = rebuilt;
        if let Some(r) = Arc::get_mut(&mut rebuilt) {
            if let TypeData::Object(obj) = &mut r.data {
                obj.structured.members = new_members;
                obj.structured.properties = new_properties;
            }
        }
        rebuilt
    }

    fn instantiated_member_source_type(&mut self, member: &Arc<Symbol>) -> Option<Arc<Type>> {
        if let Some(t) = self
            .value_symbol_links
            .get(member)
            .and_then(|l| l.resolved_type.clone())
        {
            return Some(t);
        }
        if self.is_resolving(
            Arc::as_ptr(member) as *const Symbol,
            crate::checker::TypeResolutionProperty::Type,
        ) {
            return None;
        }
        let declared = member.declarations.first().cloned();
        match declared {
            Some(decl) if matches!(
                decl.kind,
                SyntaxKind::PropertyDeclaration
                    | SyntaxKind::PropertySignature
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::MethodSignature
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
            ) => Some(self.get_type_of_symbol(member)),
            _ => None,
        }
    }
}
