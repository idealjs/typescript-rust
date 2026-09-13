#![allow(unused_imports)]

use crate::checker::checker_symbol_types::*;

impl Checker {
    pub(crate) fn attach_class_statics(&mut self, ctor_type: &Arc<Type>, node: &Arc<Node>) {
        let node_id = node.id();
        if self.class_statics_resolution_stack.contains(&node_id)
            || self.class_statics_resolution_stack.len() >= 200
        {
            return;
        }
        self.class_statics_resolution_stack.push(node_id);
        let mut members = SymbolTable::new();
        let mut properties: Vec<Arc<Symbol>> = Vec::new();

        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let mut statics: Vec<(String, Arc<Symbol>)> = Vec::new();
            for sym in class_sym.members.entries.values() {
                if sym
                    .declarations
                    .iter()
                    .any(|d| d.has_syntactic_modifier(ModifierFlags::Static))
                {
                    statics.push((sym.name.clone(), Arc::clone(sym)));
                }
            }
            for sym in class_sym.exports.entries.values() {
                if (sym
                    .declarations
                    .iter()
                    .any(|d| d.has_syntactic_modifier(ModifierFlags::Static))
                    || sym.flags.contains(SymbolFlags::Prototype))
                    && !statics.iter().any(|(n, _)| *n == sym.name)
                {
                    statics.push((sym.name.clone(), Arc::clone(sym)));
                }
            }
            for (name, sym) in statics {
                properties.push(Arc::clone(&sym));
                members.insert(name, sym);
            }
        }

        let class_members: Option<Arc<NodeList>> = match &node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(d) => Some(Arc::clone(&d.members)),
            tsox_frontend::ast::NodeData::ClassExpression(d) => Some(Arc::clone(&d.members)),
            _ => None,
        };
        if let Some(member_list) = class_members {
            for member in member_list.iter() {
                if !member.has_syntactic_modifier(ModifierFlags::Static) {
                    continue;
                }
                let Some(name_node) = member.name() else {
                    continue;
                };
                let name = name_node.text().to_string();
                if name.is_empty() || members.get(&name).is_some() {
                    continue;
                }
                let flags = match member.kind {
                    SyntaxKind::MethodDeclaration => SymbolFlags::Method,
                    SyntaxKind::GetAccessor => SymbolFlags::GetAccessor,
                    SyntaxKind::SetAccessor => SymbolFlags::SetAccessor,
                    _ => SymbolFlags::Property,
                };
                let mut sym = Symbol::new(flags, name.clone());
                sym.declarations.push(Arc::clone(member));
                let sym = Arc::new(sym);

                if let tsox_frontend::ast::NodeData::PropertyDeclaration(pd) = &member.data
                    && let Some(tn) = &pd.type_node
                {
                    let t = self.get_type_from_type_node(tn);
                    self.value_symbol_links.insert(
                        &sym,
                        crate::checker::types::ValueSymbolLinks {
                            resolved_type: Some(t),
                            ..Default::default()
                        },
                    );
                }
                properties.push(Arc::clone(&sym));
                members.insert(name, sym);
            }
        }

        let base_ctor: Option<Arc<Type>> = if let Some((base_node, _)) = self.extends_base_of(node) {
            Some(self.get_type_of_class_declaration(&base_node))
        } else if let Some(expr) = self.extends_expression_of(node) {
            // Go resolveBaseTypesOfClass：基构造类型 = checkExpression(extends 表达式)，
            // import 别名/限定名基类（clodule export =）的静态成员随值类型继承
            Some(self.get_widened_type_of_expression(&expr))
        } else {
            None
        };
        if let Some(base_ctor) = base_ctor
            && let Some(base_structured) = base_ctor.as_structured()
        {
            for (name, sym) in base_structured.members.iter() {
                if members.get(name).is_none() {
                    members.insert(name.clone(), Arc::clone(sym));
                }
            }
            for prop in &base_structured.properties {
                let name = prop.name.clone();
                if members.get(&name).is_some()
                    && !properties.iter().any(|p| Arc::ptr_eq(p, prop))
                {
                    properties.push(Arc::clone(prop));
                }
            }
        }
        self.class_statics_resolution_stack.pop();
        if members.is_empty() {
            return;
        }
        let t_mut = Arc::as_ptr(ctor_type) as *mut crate::checker::types::Type;
        unsafe {
            if let TypeData::Object(obj) = &mut (*t_mut).data {
                obj.structured.members = members;
                obj.structured.properties = properties;
            }
        }
    }

    pub(crate) fn extends_element_of(&self, class_node: &Arc<Node>) -> Option<Arc<Node>> {
        let heritage = match &class_node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(data) => data.heritage_clauses.clone(),
            tsox_frontend::ast::NodeData::ClassExpression(data) => data.heritage_clauses.clone(),
            _ => return None,
        };
        heritage?.iter().find_map(|clause| {
            if let tsox_frontend::ast::NodeData::HeritageClause(hc) = &clause.data {
                if hc.token == SyntaxKind::ExtendsKeyword {
                    return hc.types.iter().next().cloned();
                }
            }
            None
        })
    }

    pub(crate) fn extends_expression_of(&self, class_node: &Arc<Node>) -> Option<Arc<Node>> {
        let element = self.extends_element_of(class_node)?;
        match &element.data {
            tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(data) => {
                Some(Arc::clone(&data.expression))
            }
            _ => None,
        }
    }
}
