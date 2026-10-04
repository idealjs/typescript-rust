#![allow(unused_imports)]
#![allow(dead_code)]

pub(crate) use super::m1d::r21k7_defs::*;

use super::wc1b::every_type;
use crate::checker::mapper::prepend_type_mapping;
use crate::checker::checker_this_container::get_this_parameter;
use crate::checker::utilities_is_private_within_ambient::get_set_accessor_value_parameter;
use tsox_frontend::ast::mig::m3b::parameters;

use crate::checker::checker::*;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::{Node, NodeData, Symbol, SymbolFlags, SyntaxKind};
use tsox_frontend::ast::{
    is_in_js_file, is_get_accessor_declaration, is_interface_declaration,
    is_variable_declaration, is_function_expression_or_arrow_function, is_string_literal_like,
};
use tsox_frontend::ast::mig::m3b::{parameters as node_parameters, is_type_or_js_type_alias_declaration};
use crate::checker::checker_heritage_retry_limit::TypeResolutionProperty;
use tsox_frontend::ast::mig::m3c::type_arguments as node_type_arguments;

pub(crate) fn get_effective_set_accessor_type_annotation_node(
    node: &Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_effective_set_accessor_type_annotation_node"); 
    let param = get_set_accessor_value_parameter(node);
    if let Some(param) = param {
        return param.type_node().cloned();
    }
    None
}

impl Checker {
    pub(crate) fn get_annotated_accessor_this_parameter(
        &mut self,
        accessor: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_annotated_accessor_this_parameter"); 
        let parameter = self.get_accessor_this_parameter(accessor);
        parameter.and_then(|p| self.symbol_of_node(&p))
    }

    pub(crate) fn get_accessor_this_parameter(
        &mut self,
        accessor: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_accessor_this_parameter"); 
        let expected = if is_get_accessor_declaration(accessor) {
            1
        } else {
            2
        };
        if node_parameters(accessor).len() == expected {
            return get_this_parameter(accessor);
        }
        None
    }

    pub(crate) fn get_annotated_accessor_type(
        &mut self,
        accessor: &Arc<Node>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_annotated_accessor_type"); 
        let node = self.get_annotated_accessor_type_node(accessor);
        if let Some(node) = node {
            return Some(self.get_type_from_type_node(&node));
        }
        None
    }

    pub(crate) fn get_annotated_accessor_type_node(
        &mut self,
        accessor: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_annotated_accessor_type_node"); 
        match accessor.kind {
            SyntaxKind::GetAccessor | SyntaxKind::PropertyDeclaration => accessor.type_node().cloned(),
            SyntaxKind::SetAccessor => get_effective_set_accessor_type_annotation_node(accessor),
            _ => None,
        }
    }

    pub(crate) fn find_mixins(&mut self, types: &[Arc<Type>]) -> (Vec<bool>, usize) { ::tsox_core::fntrace::enter("find_mixins"); 
        let mut mixin_flags: Vec<bool> = Vec::with_capacity(types.len());
        for t in types {
            mixin_flags.push(self.is_mixin_constructor_type(t));
        }
        let mut constructor_type_count = 0;
        let mut mixin_count = 0;
        let mut first_mixin_index: Option<usize> = None;
        for (i, t) in types.iter().enumerate() {
            if !self
                .get_signatures_of_type(t, SignatureKind::Construct)
                .is_empty()
            {
                constructor_type_count += 1;
            }
            if mixin_flags[i] {
                if first_mixin_index.is_none() {
                    first_mixin_index = Some(i);
                }
                mixin_count += 1;
            }
        }
        if constructor_type_count > 0 && constructor_type_count == mixin_count {
            mixin_flags[first_mixin_index.unwrap()] = false;
            mixin_count -= 1;
        }
        (mixin_flags, mixin_count)
    }

    pub(crate) fn get_apparent_type_of_mapped_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_apparent_type_of_mapped_type"); 
        if let Some(cached) = t.resolved_apparent_type_of_mapped_type() {
            return cached;
        }
        let result = self.get_resolved_apparent_type_of_mapped_type(t);
        t.set_resolved_apparent_type_of_mapped_type(&result);
        result
    }

    pub(crate) fn get_apparent_type_of_intersection_type(
        &mut self,
        t: &Arc<Type>,
        this_argument: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_apparent_type_of_intersection_type"); 
        if t.id == this_argument.id {
            if let Some(cached) = t.resolved_apparent_type_of_intersection() {
                return cached;
            }
            let result = self.get_type_with_this_argument(t, Some(this_argument), true);
            t.set_resolved_apparent_type_of_intersection(&result);
            return result;
        }
        self.get_type_with_this_argument(t, Some(this_argument), true)
    }

    pub(crate) fn get_effective_type_arguments(
        &mut self,
        node: &Arc<Node>,
        type_parameters: &[Arc<Type>],
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_effective_type_arguments"); 
        let type_arguments: Vec<Arc<Type>> = node_type_arguments(node)
            .iter()
            .map(|n| self.get_type_from_type_node(n))
            .collect();
        let min_type_argument_count = self.get_min_type_argument_count(type_parameters);
        self.fill_missing_type_arguments(
            &type_arguments,
            type_parameters,
            min_type_argument_count,
            is_in_js_file(node),
        )
    }

    pub(crate) fn get_default_type_argument_type(
        &mut self,
        is_in_javascript_file: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_default_type_argument_type"); 
        if is_in_javascript_file {
            return self.get_any_type();
        }
        self.unknown_type()
    }

    pub(crate) fn get_default_or_unknown_from_type_parameter(
        &mut self,
        t: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_default_or_unknown_from_type_parameter"); 
        self.get_default_from_type_parameter(t)
            .unwrap_or_else(|| self.unknown_type())
    }

    pub(crate) fn get_constraint_declaration_for_mapped_type(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_constraint_declaration_for_mapped_type"); 
        let m = t.mapped_data()?;
        let declaration = m.declaration.as_ref()?;
        if let NodeData::MappedTypeNode(d) = &declaration.data {
            if let NodeData::TypeParameterDeclaration(tp) = &d.type_parameter.data {
                return tp.constraint.clone();
            }
        }
        None
    }

    pub(crate) fn for_each_mapped_type_property_key_type_and_index_signature_key_type(
        &mut self,
        t: &Arc<Type>,
        include: TypeFlags,
        strings_only: bool,
        cb: &mut dyn FnMut(&Arc<Type>),
    ) { ::tsox_core::fntrace::enter("for_each_mapped_type_property_key_type_and_index_signature_key_type"); 
        for prop in self.get_properties_of_type(t) {
            let key = self.get_literal_type_from_property(&prop);
            cb(&key);
        }
        if t.flags.contains(TypeFlags::Any) {
            cb(&self.string_type());
        } else {
            for info in self.get_index_infos_of_type(t) {
                if !strings_only {
                    if let Some(key_type) = &info.key_type {
                        cb(key_type);
                    }
                } else if let Some(key_type) = &info.key_type {
                    if key_type
                        .flags
                        .intersects(TypeFlags::String | TypeFlags::TemplateLiteral)
                    {
                        cb(key_type);
                    }
                }
            }
        }
    }

    pub(crate) fn get_class_or_interface_like_declaration(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_class_or_interface_like_declaration"); 
        if symbol.flags.intersects(SymbolFlags::Class | SymbolFlags::Function) {
            return symbol.value_declaration.clone();
        }
        symbol.declarations.iter().find(|d| {
            if is_interface_declaration(d) {
                return true;
            }
            if !is_variable_declaration(d) {
                return false;
            }
            match tsox_frontend::ast::mig::m3b::initializer(d) {
                Some(initializer) => is_function_expression_or_arrow_function(initializer),
                None => false,
            }
        }).cloned()
    }

    pub(crate) fn get_declared_type_of_type_parameter(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_declared_type_of_type_parameter"); 
        if let Some(t) = self
            .declared_type_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return t;
        }
        let t = self.new_type_parameter(Some(Arc::clone(symbol)));
        self.declared_type_links
            .get_or_default(symbol)
            .declared_type = Some(Arc::clone(&t));
        t
    }

    pub(crate) fn get_declared_type_of_type_alias(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_declared_type_of_type_alias"); 
        if let Some(t) = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return t;
        }
        if !self.push_type_resolution(Arc::as_ptr(symbol), TypeResolutionProperty::DeclaredType) {
            return self.error_type();
        }
        let declaration = symbol
            .declarations
            .iter()
            .find(|d| is_type_or_js_type_alias_declaration(d))
            .cloned();
        // Go declaration.Type()（checker.go:25261）：泛型 type_node() 访问器无
        // TypeAliasDeclaration 臂恒返 None，按别名声明 data 直取体类型节点
        let type_node = declaration.as_ref().and_then(|d| match &d.data {
            NodeData::TypeAliasDeclaration(ta) => Some(Arc::clone(&ta.type_node)),
            _ => None,
        });
        let t = match type_node {
            Some(type_node) => self.get_type_from_type_node(&type_node),
            None => self.error_type(),
        };
        if self.pop_type_resolution() {
            let type_parameters =
                self.get_local_type_parameters_of_class_or_interface_or_type_alias(symbol);
            if !type_parameters.is_empty() {
                self.type_alias_links.get_or_default(symbol).type_parameters =
                    type_parameters;
            }
            let final_t = if t.id == self.intrinsic_marker_type().id
                && symbol.name == "BuiltinIteratorReturn"
            {
                self.get_builtin_iterator_return_type()
            } else {
                t
            };
            self.type_alias_links
                .get_or_default(symbol)
                .declared_type = Some(Arc::clone(&final_t));
            return final_t;
        }
        self.error_type()
    }

    pub(crate) fn get_emit_syntax_for_module_specifier_expression(
        &mut self,
        _usage: &Arc<Node>,
    ) -> tsox_core::core::compiler_options_kinds::ResolutionMode { ::tsox_core::fntrace::enter("get_emit_syntax_for_module_specifier_expression"); 
        tsox_core::core::compiler_options_kinds::ResolutionMode::None
    }
}
