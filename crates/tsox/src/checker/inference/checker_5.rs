#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn iife_contextual_signature(
        &mut self,
        node: &Arc<crate::ast::Node>,
    ) -> Option<Arc<Signature>> {
        let mut parent = node.parent.clone()?;
        while parent.kind == SyntaxKind::ParenthesizedExpression {
            parent = parent.parent.clone()?;
        }
        let crate::ast::NodeData::CallExpression(call) = &parent.data else {
            return None;
        };
        if !Arc::ptr_eq(&call.expression, node) {
            return None;
        }
        let arg_count = call.arguments.len();
        if arg_count == 0 {
            return None;
        }

        let key = node.id();
        if !self.resolving_function_like.insert(key) {
            return None;
        }
        let mut params: Vec<Arc<crate::ast::Symbol>> = Vec::with_capacity(arg_count);
        for (i, arg) in call.arguments.iter().enumerate() {
            let t = self.get_type_of_node(arg);
            let sym = Arc::new(crate::ast::Symbol::new(
                crate::ast::SymbolFlags::Property,
                format!("__iife{}", i),
            ));
            self.value_symbol_links.insert(
                &sym,
                ValueSymbolLinks {
                    resolved_type: Some(t),
                    ..Default::default()
                },
            );
            params.push(sym);
        }
        self.resolving_function_like.remove(&key);
        Some(Arc::new(Signature {
            id: 0,
            flags: SignatureFlags::None,
            min_argument_count: 0,
            resolved_min_argument_count: -1,
            declaration: None,
            type_parameters: Vec::new(),
            parameters: params,
            this_parameter: None,
            resolved_return_type: std::sync::OnceLock::new(),
            resolved_type_predicate: None,
            target: None,
            mapper: None,
            isolated_signature_type: std::sync::OnceLock::new(),
            instantiated_parameter_types: None,
        }))
    }

    pub(crate) fn get_contextual_type_for_initializer_expression(
        &mut self,
        node: &Arc<crate::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        let declaration = node.parent.as_ref()?;

        let is_initializer = match &declaration.data {
            NodeData::VariableDeclaration(data) => data
                .initializer
                .as_ref()
                .map_or(false, |init| Arc::ptr_eq(init, node)),
            NodeData::ParameterDeclaration(data) => data
                .initializer
                .as_ref()
                .map_or(false, |init| Arc::ptr_eq(init, node)),
            NodeData::PropertyDeclaration(data) => data
                .initializer
                .as_ref()
                .map_or(false, |init| Arc::ptr_eq(init, node)),
            NodeData::BindingElement(data) => data
                .initializer
                .as_ref()
                .map_or(false, |init| Arc::ptr_eq(init, node)),
            _ => false,
        };
        if !is_initializer {
            return None;
        }

        let type_node = match &declaration.data {
            NodeData::VariableDeclaration(data) => data.type_node.as_ref(),
            NodeData::ParameterDeclaration(data) => data.type_node.as_ref(),
            NodeData::PropertyDeclaration(data) => data.type_node.as_ref(),
            NodeData::PropertySignatureDeclaration(data) => Some(&data.type_node),
            NodeData::BindingElement(_) => None,
            _ => None,
        };

        if let Some(type_node) = type_node {
            return Some(self.get_type_from_type_node(type_node));
        }

        if let NodeData::BindingElement(_) = &declaration.data {
            if let Some(ctx) = self.get_contextual_type_from_binding_element(declaration) {
                return Some(ctx);
            }
        }

        None
    }

    pub(crate) fn get_contextual_type_from_binding_element(
        &mut self,
        binding_element: &Arc<crate::ast::Node>,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        let binding_pattern = binding_element.parent.as_ref()?;
        let var_declaration = binding_pattern.parent.as_ref()?;

        let var_data = match &var_declaration.data {
            NodeData::VariableDeclaration(d) => d,
            _ => return None,
        };

        let initializer = var_data.initializer.as_ref()?;
        let init_type = self.get_type_of_node(initializer);

        match binding_pattern.kind {
            SyntaxKind::ArrayBindingPattern => {
                let elements = match &binding_pattern.data {
                    NodeData::BindingPattern(d) => &d.elements,
                    _ => return None,
                };
                let idx = elements
                    .nodes
                    .iter()
                    .position(|e| Arc::ptr_eq(e, binding_element))?;

                self.get_element_type_of_array(&init_type, idx)
            }
            SyntaxKind::ObjectBindingPattern => {
                let be_data = match &binding_element.data {
                    NodeData::BindingElement(d) => d,
                    _ => return None,
                };
                let property_name = be_data.property_name.as_ref().unwrap_or_else(|| {
                    be_data
                        .name
                        .as_ref()
                        .expect("binding element has name or property_name")
                });
                let name = property_name.text();
                self.property_type_of_type(&init_type, &name)
            }
            _ => None,
        }
    }

    pub(crate) fn get_element_type_of_array(&mut self, t: &Arc<Type>, index: usize) -> Option<Arc<Type>> {
        if let TypeData::Tuple(tuple) = &t.data {
            if index < tuple.element_infos.len() {
                if let Some(ref elem) = tuple.element_infos[index].type_ {
                    return Some(Arc::clone(elem));
                }
            }
        }

        if self.is_array_type(t) || matches!(t.data, TypeData::EvolvingArray(_)) {
            return Some(self.get_array_element_type(t));
        }
        None
    }

    pub(crate) fn property_type_of_type(&mut self, t: &Arc<Type>, name: &str) -> Option<Arc<Type>> {
        let prop = self.get_property_of_type(t, name)?;
        Some(self.get_type_of_symbol(&prop))
    }

    pub(crate) fn get_contextual_type_for_call_or_new(
        &mut self,
        node: &crate::ast::Node,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        let parent = node.parent.as_ref()?;
        match &parent.data {
            NodeData::VariableDeclaration(data) => data
                .type_node
                .as_ref()
                .map(|tn| self.get_type_from_type_node(tn)),
            NodeData::ReturnStatement(_) => {
                let fn_node = parent.parent.as_ref()?;
                self.get_return_type_annotation_of_function(fn_node)
            }
            _ => None,
        }
    }

    pub(crate) fn get_return_type_annotation_of_function(
        &mut self,
        node: &crate::ast::Node,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        let type_node = match &node.data {
            NodeData::FunctionDeclaration(d) => d.type_node.as_ref(),
            NodeData::FunctionExpression(d) => d.type_node.as_ref(),
            NodeData::ArrowFunction(d) => d.type_node.as_ref(),
            NodeData::MethodDeclaration(d) => d.type_node.as_ref(),
            NodeData::MethodSignatureDeclaration(d) => d.type_node.as_ref(),
            NodeData::GetAccessorDeclaration(d) => d.type_node.as_ref(),
            NodeData::SetAccessorDeclaration(_) => None,
            _ => None,
        };
        type_node.map(|tn| self.get_type_from_type_node(tn))
    }

    pub(crate) fn get_contextual_type_for_return_expression(
        &mut self,
        _node: &Arc<crate::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        let mut current = _node.parent.as_ref()?.clone();
        loop {
            match current.kind {
                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::Constructor
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => break,
                SyntaxKind::SourceFile => return None,
                _ => {
                    current = current.parent.as_ref()?.clone();
                }
            }
        }
        self.contextual_return_type_of(&current)
    }

    pub fn contextual_return_type_of(
        &mut self,
        fn_node: &Arc<crate::ast::Node>,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        let type_node = match &fn_node.data {
            NodeData::FunctionDeclaration(data) => data.type_node.clone(),
            NodeData::FunctionExpression(data) => data.type_node.clone(),
            NodeData::ArrowFunction(data) => data.type_node.clone(),
            NodeData::MethodDeclaration(data) => data.type_node.clone(),
            NodeData::ConstructorDeclaration(data) => data.type_node.clone(),
            NodeData::GetAccessorDeclaration(data) => data.type_node.clone(),
            NodeData::SetAccessorDeclaration(data) => data.type_node.clone(),
            _ => None,
        };
        if let Some(type_node) = type_node {
            return Some(self.get_type_from_type_node(&type_node));
        }

        if let Some(signature) = self.get_contextual_signature(fn_node) {
            if let Some(return_type) = self.get_return_type_of_signature(&signature) {
                return Some(return_type);
            }
        }

        let mut parent = fn_node.parent.clone()?;
        while parent.kind == SyntaxKind::ParenthesizedExpression {
            parent = parent.parent.clone()?;
        }
        if let NodeData::CallExpression(call) = &parent.data {
            if Arc::ptr_eq(&call.expression, fn_node) {
                return self.get_contextual_type(&parent, ContextFlags::None);
            }
        }
        None
    }

}
