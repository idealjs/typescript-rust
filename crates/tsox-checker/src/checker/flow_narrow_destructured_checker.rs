#![allow(unused_imports)]

use crate::checker::flow_narrow_destructured::*;

use tsox_frontend::ast::ElementAccessExpressionData;
use tsox_frontend::ast::StringLiteralData;

impl Checker {
    pub(crate) fn binding_pattern_sibling_access(
        &self,
        expr: &Arc<Node>,
        pattern: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        if expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let symbol = self.resolve_identifier(expr)?;
        let declaration = symbol.value_declaration.as_ref()?;
        if declaration.kind != SyntaxKind::BindingElement {
            return None;
        }
        let NodeData::BindingElement(be) = &declaration.data else {
            return None;
        };
        if be.initializer.is_some() || be.dot_dot_dot_token.is_some() {
            return None;
        }
        if !Arc::ptr_eq(declaration.parent().as_ref()?, pattern) {
            return None;
        }
        Some(Arc::clone(declaration))
    }

    pub(crate) fn narrow_destructured_symbol_type(
        &mut self,
        symbol: &Arc<Symbol>,
        location: &Arc<Node>,
        location_flow: &Arc<FlowNode>,
    ) -> Option<Arc<Type>> {
        let decl = Arc::clone(symbol.value_declaration.as_ref()?);
        if !matches!(decl.kind, SyntaxKind::BindingElement) {
            return None;
        }
        let NodeData::BindingElement(be) = &decl.data else {
            return None;
        };
        if be.initializer.is_some() || be.dot_dot_dot_token.is_some() {
            return None;
        }
        let pattern = Arc::clone(decl.parent().as_ref()?);
        let pattern_parent = Arc::clone(pattern.parent().as_ref()?);
        let NodeData::BindingPattern(pd) = &pattern.data else {
            return None;
        };
        if pd.elements.nodes.len() < 2 {
            return None;
        }
        let root = Self::root_declaration(&decl)?;
        let is_const_variable = matches!(root.data, NodeData::VariableDeclaration(_))
            && root
                .parent()
                .as_ref()
                .is_some_and(|p| p.flags.contains(tsox_frontend::ast::NodeFlags::Const));
        let is_parameter = matches!(root.data, NodeData::ParameterDeclaration(_));
        if !is_const_variable && !is_parameter {
            return None;
        }
        if let NodeData::VariableDeclaration(rd) = &root.data
            && let Some(root_initializer) = &rd.initializer
            && tsox_frontend::ast::utilities::is_node_descendant_of(location, root_initializer)
        {
            return None;
        }
        let parent_type = self.type_for_binding_pattern_parent(&root)?;
        let constraint = self.constituents_base_constraint_union(&parent_type);
        let guard_id = root.id();
        if self.binding_pattern_narrowing_stack.contains(&guard_id) {
            return None;
        }
        let is_nested = matches!(&pattern_parent.data, NodeData::BindingElement(_));
        if is_nested || !constraint.flags.contains(TypeFlags::Union) {
            self.binding_pattern_narrowing_stack.push(guard_id);
            let result = self.narrow_destructured_by_element_access(
                &decl,
                &root,
                &parent_type,
                location_flow,
            );
            self.binding_pattern_narrowing_stack.pop();
            return result;
        }
        self.binding_pattern_narrowing_stack.push(guard_id);
        let result = self.narrow_binding_pattern_reference(&pattern, location_flow, &constraint);
        self.binding_pattern_narrowing_stack.pop();
        let narrowed = result?;
        if narrowed.flags.contains(TypeFlags::Never) {
            return Some(self.never_type());
        }
        self.binding_element_type_from_parent_type(&decl, &pattern_parent, &narrowed)
    }

    fn narrow_destructured_by_element_access(
        &mut self,
        decl: &Arc<Node>,
        pattern_parent: &Arc<Node>,
        parent_type: &Arc<Type>,
        location_flow: &Arc<FlowNode>,
    ) -> Option<Arc<Type>> {
        if parent_type.flags.intersects(TypeFlags::Any | TypeFlags::Never) {
            return Some(Arc::clone(parent_type));
        }
        let parent_type = self.filter_binding_parent_undefined(pattern_parent, Arc::clone(parent_type));
        let mut chain: Vec<String> = Vec::new();
        let mut element = Arc::clone(decl);
        loop {
            let pattern = element.parent()?;
            if pattern.kind != SyntaxKind::ObjectBindingPattern {
                return None;
            }
            chain.push(Checker::binding_element_property_name(&element)?);
            let up = pattern.parent()?;
            if !matches!(up.data, NodeData::BindingElement(_)) {
                break;
            }
            element = up;
        }
        let initializer = match &pattern_parent.data {
            NodeData::VariableDeclaration(d) => d.initializer.clone(),
            NodeData::ParameterDeclaration(d) => d.initializer.clone(),
            _ => None,
        }?;
        let mut expr = initializer;
        for name in chain.iter().rev() {
            let literal = Arc::new(Node::new(
                SyntaxKind::StringLiteral,
                NodeData::StringLiteral(StringLiteralData {
                    text: name.clone(),
                    token_flags: 0,
                }),
            ));
            let access = Arc::new(Node::new(
                SyntaxKind::ElementAccessExpression,
                NodeData::ElementAccessExpression(ElementAccessExpressionData {
                    expression: expr,
                    question_dot_token: None,
                    argument_expression: Arc::clone(&literal),
                }),
            ));
            literal.set_parent(&access);
            expr = access;
        }
        let mut declared = parent_type;
        for name in chain.iter().rev() {
            declared = self.get_property_type_of_type(&declared, name)?;
        }
        let target = FlowRef::Node(Arc::clone(&expr));
        let key = self.flow_cache_key(&target, location_flow, &declared);
        if let Some(cached) = self.flow_type_cache.get(&key) {
            return Some(Arc::clone(cached));
        }
        self.flow_type_cache.insert(key, Arc::clone(&declared));
        let mut query = FlowQuery::default();
        let narrowed =
            self.type_at_flow_node(&declared, &declared, location_flow, &target, 0, &mut query);
        self.flow_type_cache.insert(key, Arc::clone(&narrowed));
        Some(narrowed)
    }

    fn narrow_binding_pattern_reference(
        &mut self,
        pattern: &Arc<Node>,
        location_flow: &Arc<FlowNode>,
        constraint: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let target = FlowRef::Node(Arc::clone(pattern));
        let key = self.flow_cache_key(&target, location_flow, constraint);
        if let Some(cached) = self.flow_type_cache.get(&key) {
            return Some(Arc::clone(cached));
        }
        self.flow_type_cache.insert(key, Arc::clone(constraint));
        let mut query = FlowQuery::default();
        let narrowed =
            self.type_at_flow_node(constraint, constraint, location_flow, &target, 0, &mut query);
        self.flow_type_cache.insert(key, Arc::clone(&narrowed));
        Some(narrowed)
    }

    fn type_for_binding_pattern_parent(&mut self, parent: &Arc<Node>) -> Option<Arc<Type>> {
        let (type_node, initializer) = match &parent.data {
            NodeData::VariableDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            NodeData::ParameterDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            _ => return None,
        };
        if let Some(tn) = type_node {
            return Some(self.get_type_from_type_node(&tn));
        }
        let _ = initializer;
        match &parent.data {
            NodeData::ParameterDeclaration(_) => self.contextual_type_of_parameter(parent),
            _ => self.initial_type_of_declaration(parent),
        }
    }

    fn constituents_base_constraint_union(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(t);
        if !constituents.iter().any(|c| c.flags.intersects(TYPE_FLAGS_INSTANTIABLE)) {
            return Arc::clone(t);
        }
        let mapped: Vec<Arc<Type>> = constituents
            .into_iter()
            .map(|c| {
                self.get_base_constraint_of_type(&c).unwrap_or(c)
            })
            .collect();
        if mapped.len() == 1 {
            return mapped.into_iter().next().expect("exactly one");
        }
        self.get_union_type(mapped)
    }

    fn binding_element_type_from_parent_type(
        &mut self,
        decl: &Arc<Node>,
        pattern_parent: &Arc<Node>,
        parent_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        if parent_type.flags.intersects(TypeFlags::Any | TypeFlags::Never) {
            return Some(Arc::clone(parent_type));
        }
        let pattern = Arc::clone(decl.parent().as_ref()?);
        let NodeData::BindingElement(be) = &decl.data else {
            return None;
        };
        let t: Option<Arc<Type>> = match pattern.kind {
            SyntaxKind::ObjectBindingPattern => {
                let name = Checker::binding_element_property_name(decl)?;
                Some(
                    self.get_property_type_of_type(parent_type, &name)
                        .unwrap_or_else(|| self.error_type()),
                )
            }
            SyntaxKind::ArrayBindingPattern if be.dot_dot_dot_token.is_none() => {
                let index = Checker::binding_element_index(&pattern, decl)?;
                self.destructured_array_element_type(parent_type, index, None)
            }
            _ => None,
        };
        Some(t?)
    }
}
