#![allow(unused_imports)]

use crate::checker::jsx_impl_chunk_2::*;

impl Checker {
    pub fn get_jsx_element_properties_name(
        &self,
        _jsx_namespace: &Arc<tsox_frontend::ast::Symbol>,
    ) -> Option<String> {
        None
    }

    pub fn get_jsx_element_children_property_name(
        &self,
        jsx_namespace: &Arc<tsox_frontend::ast::Symbol>,
    ) -> Option<String> {
        // Go getJsxElementChildrenPropertyName：react-jsx 模式固定 'children'
        if matches!(
            self.compiler_options.jsx,
            tsox_core::core::compiler_options::JsxEmit::ReactJSX
                | tsox_core::core::compiler_options::JsxEmit::ReactJSXDev
        ) {
            return Some("children".to_string());
        }
        self.get_name_from_jsx_element_attributes_container(
            crate::checker::jsx_impl_chunk::JsxNames::ELEMENT_CHILDREN_ATTRIBUTE_NAME_CONTAINER,
            jsx_namespace,
        )
    }

    pub fn get_name_from_jsx_element_attributes_container(
        &self,
        name_of_attrib_prop_container: &str,
        jsx_namespace: &Arc<tsox_frontend::ast::Symbol>,
    ) -> Option<String> {
        // Go getNameFromJsxElementAttributesContainer：JSX 命名空间导出的
        // 容器接口（ElementChildrenAttribute 等）的唯一成员名
        let container = jsx_namespace
            .exports
            .get(name_of_attrib_prop_container)
            .or_else(|| jsx_namespace.members.get(name_of_attrib_prop_container))?;
        let mut names = container.members.entries.keys().cloned();
        let first = names.next()?;
        if names.next().is_some() {
            return None;
        }
        Some(first)
    }

    pub fn get_static_type_of_referenced_jsx_constructor(
        &mut self,
        _context: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_intrinsic_attributes_type_from_string_literal_type(
        &mut self,
        _t: &Arc<crate::checker::types::Type>,
        _location: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_reference_kind(&self, _node: &Arc<Node>) -> JsxReferenceKind {
        JsxReferenceKind::Function
    }

    pub fn create_signature_for_jsx_intrinsic(
        &mut self,
        _node: &Arc<Node>,
        _result: &Arc<crate::checker::types::Type>,
    ) -> Option<Arc<crate::checker::types::Signature>> {
        None
    }

    pub fn get_intrinsic_attributes_type_from_jsx_opening_like_element(
        &mut self,
        _node: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_intrinsic_tag_symbol(
        &self,
        _node: &Arc<Node>,
    ) -> Option<Arc<tsox_frontend::ast::Symbol>> {
        None
    }

    pub fn get_jsx_stateless_element_type_at(
        &mut self,
        _location: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_element_class_type_at(
        &mut self,
        _location: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_element_type_at(
        &mut self,
        _location: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_element_type_type_at(
        &mut self,
        _location: &Arc<Node>,
    ) -> Option<Arc<crate::checker::types::Type>> {
        None
    }

    pub fn get_jsx_namespace_str(&self, _location: &Arc<Node>) -> String {
        "jsx".to_string()
    }

    pub fn get_local_jsx_namespace(&self, _file: &Arc<tsox_frontend::ast::SourceFile>) -> String {
        "jsx".to_string()
    }

    pub fn get_jsx_factory_entity(&self, _location: &Arc<Node>) -> Option<Arc<Node>> {
        None
    }

    pub fn get_jsx_fragment_factory_entity(&self, _location: &Arc<Node>) -> Option<Arc<Node>> {
        None
    }

    pub fn get_jsx_namespace_container_for_implicit_import(
        &self,
        _location: &Arc<Node>,
    ) -> Option<Arc<tsox_frontend::ast::Symbol>> {
        None
    }

    pub(crate) fn mark_jsx_alias_referenced(&mut self, opening: &Arc<Node>) {
        use tsox_core::core::compiler_options::JsxEmit;
        use tsox_frontend::ast::SymbolFlags;
        if matches!(self.compiler_options.jsx, JsxEmit::ReactJSX | JsxEmit::ReactJSXDev) {
            return;
        }
        let is_fragment = matches!(opening.kind, SyntaxKind::JsxOpeningFragment);
        let namespace = self.jsx_mark_namespace(is_fragment);
        if !(is_fragment && namespace == "null") {
            if let Some(sym) = self.jsx_factory_namespace_symbol(&namespace) {
                self.record_symbol_reference(&sym, SymbolFlags::all());
            }
        }
        if is_fragment {
            let element_ns = self.jsx_mark_namespace(false);
            if let Some(sym) = self.jsx_factory_namespace_symbol(&element_ns) {
                self.record_symbol_reference(&sym, SymbolFlags::VALUE);
            }
        }
    }

    fn jsx_mark_namespace(&self, is_fragment: bool) -> String {
        let pragma_first = |pragma: &str| {
            self.local_jsx_pragma_factory(pragma)
                .and_then(|f| f.split('.').next().map(str::to_string))
                .filter(|s| !s.is_empty())
        };
        if is_fragment {
            if let Some(ns) = pragma_first("jsxfrag") {
                return ns;
            }
            let opt_frag = self
                .compiler_options
                .jsx_fragment_factory
                .split('.')
                .next()
                .unwrap_or("");
            if !opt_frag.is_empty() {
                return opt_frag.to_string();
            }
        } else if let Some(ns) = pragma_first("jsx") {
            return ns;
        }
        let opt_factory = self
            .compiler_options
            .jsx_factory
            .split('.')
            .next()
            .unwrap_or("");
        if !opt_factory.is_empty() {
            return opt_factory.to_string();
        }
        let ns = self.compiler_options.react_namespace.as_str();
        if ns.is_empty() {
            "React".to_string()
        } else {
            ns.to_string()
        }
    }

    pub(crate) fn jsx_factory_namespace_symbol(
        &self,
        name: &str,
    ) -> Option<Arc<tsox_frontend::ast::Symbol>> {
        use tsox_frontend::ast::SymbolFlags;
        let symbol_map = self.program.symbol_map();
        let value = |sym: &std::sync::Arc<tsox_frontend::ast::Symbol>| {
            if sym.flags.intersects(SymbolFlags::Alias) {
                match self.follow_alias(sym) {
                    Some(t) if std::sync::Arc::ptr_eq(&t, sym) => true,
                    Some(t) => t.flags.intersects(SymbolFlags::VALUE),
                    None => true,
                }
            } else {
                sym.flags.intersects(SymbolFlags::VALUE)
            }
        };
        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id)
                && let Some(sym) = locals.get(name)
                && value(sym)
            {
                return Some(Arc::clone(sym));
            }
            if let Some(cs) = symbol_map.symbols.get(&container_id)
                && (!cs.flags.intersects(SymbolFlags::Class)
                    || cs.flags.intersects(SymbolFlags::Function))
                && let Some(sym) = cs.members.get(name)
                && value(sym)
            {
                return Some(Arc::clone(sym));
            }
            if let Some(cs) = symbol_map.symbols.get(&container_id)
                && cs.flags.intersects(SymbolFlags::MODULE)
                && !cs.flags.intersects(SymbolFlags::Class)
                && let Some(sym) = cs.exports.get(name)
                && value(sym)
            {
                return Some(Arc::clone(sym));
            }
        }
        self.globals
            .get(name)
            .filter(|g| g.flags.intersects(SymbolFlags::VALUE))
            .cloned()
    }

    pub fn get_jsx_runtime_import_specifier(
        &self,
        _file: &Arc<tsox_frontend::ast::SourceFile>,
    ) -> (String, Option<Arc<Node>>) {
        (String::new(), None)
    }
}
