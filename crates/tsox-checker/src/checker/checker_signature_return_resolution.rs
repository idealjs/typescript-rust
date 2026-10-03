#![allow(unused_imports)]

use crate::checker::checker::*;

impl Checker {
    pub fn push_signature_return_resolution(&mut self, decl: *const Node) -> bool { ::tsox_core::fntrace::enter("push_signature_return_resolution"); 
        let cycle_start = self
            .signature_return_resolutions
            .iter()
            .rposition(|e| e.0 == decl);
        if let Some(idx) = cycle_start {
            // Go pushTypeResolution(sig, ResolvedReturnType) 只覆盖返回型解析窗口:
            // 建签期(参数/上下文签名)重入命中缓存惰性签名,不构成环;
            // 实参推断期的重入(Go checkNodeDeferred 下体推断已延后)亦不构成环;
            // 仅调用位 callee 签名解析内的重入是 getReturnTypeOfSignature 环
            if self.call_return_query_depth > 0
                && self.callee_resolution_depth > 0
                && self.signature_return_resolutions[idx].2
            {
                for e in &mut self.signature_return_resolutions[idx..] {
                    e.1 = false;
                }
            }
            return false;
        }
        self.signature_return_resolutions.push((decl, true, false));
        true
    }

    pub fn pop_signature_return_resolution(&mut self) -> bool { ::tsox_core::fntrace::enter("pop_signature_return_resolution"); 
        self.signature_return_resolutions
            .pop()
            .map(|e| e.1)
            .unwrap_or(true)
    }

    pub fn set_signature_return_inference_phase(&mut self, decl: *const Node, on: bool) { ::tsox_core::fntrace::enter("set_signature_return_inference_phase"); 
        if let Some(e) = self
            .signature_return_resolutions
            .iter_mut()
            .find(|e| e.0 == decl)
        {
            e.2 = on;
        }
    }

    pub fn is_resolving_signature_return(&self, decl: *const Node) -> bool { ::tsox_core::fntrace::enter("is_resolving_signature_return"); 
        self.signature_return_resolutions
            .iter()
            .any(|e| e.0 == decl)
    }

    pub fn partial_type_of_function_like(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("partial_type_of_function_like"); 
        let key = Arc::as_ptr(node);
        if self.partial_fn_type_builds.contains(&key) {
            return self.get_any_type();
        }
        self.partial_fn_type_builds.insert(key);
        let result = self.partial_type_of_function_like_inner(node);
        self.partial_fn_type_builds.remove(&key);
        result
    }

    fn partial_type_of_function_like_inner(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("partial_type_of_function_like_inner"); 
        let parameters = match &node.data {
            tsox_frontend::ast::NodeData::FunctionExpression(data) => &data.parameters,
            tsox_frontend::ast::NodeData::ArrowFunction(data) => &data.parameters,
            tsox_frontend::ast::NodeData::FunctionDeclaration(data) => &data.parameters,
            _ => return self.get_any_type(),
        };
        let contextual_signature = self
            .get_contextual_signature(node)
            .or_else(|| self.iife_contextual_signature(node));
        let is_arrow = matches!(node.data, tsox_frontend::ast::NodeData::ArrowFunction(_));
        if is_arrow {
            self.push_arrow_function_scope(node);
        } else {
            self.push_function_scope(node);
        }
        let sig = self.build_signature_from_function_like_type_node(
            parameters,
            self.get_any_type(),
            false,
            contextual_signature.as_ref(),
            Some(Arc::clone(node)),
        );
        if is_arrow {
            self.pop_arrow_function_scope();
        } else {
            self.pop_function_scope();
        }
        if !sig.type_parameters.is_empty()
            && let Some(contextual) = contextual_signature
            && contextual.type_parameters.is_empty()
        {
            let inst = self.instantiate_signature_in_context_of(&sig, &contextual);
            return self.create_function_or_constructor_type(vec![inst], false);
        }
        self.create_function_or_constructor_type(vec![sig], false)
    }

    pub fn report_signature_return_circularity(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("report_signature_return_circularity"); 
        let type_node = Self::function_like_return_type_annotation(node);
        let Some(file) = self.get_source_file_of_node(node).or_else(|| self.current_file.clone())
        else {
            return;
        };
        if let Some(tn) = type_node {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                Some(file),
                tn.loc,
                tsox_core::diagnostics::messages_generated::
                    RETURN_TYPE_ANNOTATION_CIRCULARLY_REFERENCES_ITSELF,
                vec![],
            ));
            return;
        }
        if self.no_implicit_any {
            let (name_node, name_text) = Self::return_circularity_name(node);
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                Some(file),
                name_node.loc,
                tsox_core::diagnostics::messages_generated::
                    X_0_IMPLICITLY_HAS_RETURN_TYPE_ANY_BECAUSE_IT_DOES_NOT_HAVE_A_RETURN_TYPE_ANNOTATION_AND_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ONE_OF_ITS_RETURN_EXPRESSIONS,
                vec![name_text],
            ));
        }
    }

    fn function_like_return_type_annotation(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("function_like_return_type_annotation"); 
        match &node.data {
            NodeData::FunctionDeclaration(d) => d.type_node.clone(),
            NodeData::FunctionExpression(d) => d.type_node.clone(),
            NodeData::ArrowFunction(d) => d.type_node.clone(),
            NodeData::MethodDeclaration(d) => d.type_node.clone(),
            NodeData::GetAccessorDeclaration(d) => d.type_node.clone(),
            NodeData::SetAccessorDeclaration(d) => d.type_node.clone(),
            _ => None,
        }
    }

    fn return_circularity_name(node: &Arc<Node>) -> (Arc<Node>, String) { ::tsox_core::fntrace::enter("return_circularity_name"); 
        if let Some(name) = node.name() {
            return (name.clone(), name.text().to_string());
        }
        let mut cur = node.parent();
        while let Some(p) = cur {
            match p.kind {
                SyntaxKind::VariableDeclaration | SyntaxKind::PropertyDeclaration => {
                    if let Some(name) = p.name() {
                        return (name.clone(), name.text().to_string());
                    }
                    return (p.clone(), String::new());
                }
                SyntaxKind::BinaryExpression => {
                    cur = p.parent();
                }
                _ => break,
            }
        }
        (node.clone(), String::new())
    }
}
