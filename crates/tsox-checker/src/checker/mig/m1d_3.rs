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
use tsox_frontend::ast::{is_require_call, is_external_module_reference, is_import_call};
use crate::checker::mig::wc1b::walk_up_parenthesized_expressions;
use crate::checker::mig::m1f::r19k9_defs::R19K9NodeExt;
use tsox_frontend::ast::mig::m3b::{parameters as node_parameters, is_type_or_js_type_alias_declaration};
use tsox_frontend::ast::mig::m3c::type_parameters as alias_type_parameters;
use crate::checker::utilities_is_optional_symbol::is_type_alias;
use tsox_frontend::ast::node_kind_is;
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
        let key = CachedTypeKey {
            kind: CachedTypeKind::ApparentType,
            type_id: this_argument.id,
        };
        if let Some(cached) = self.cached_types.get(&key) {
            return Arc::clone(cached);
        }
        let result = self.get_type_with_this_argument(t, Some(this_argument), true);
        self.cached_types.insert(key, Arc::clone(&result));
        result
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
        // Go 类型参数符号的声明型是单例对象（约束经 getConstraintOfTypeParameter
        // checker.go:18074-18099 惰性回填同一实例）；本仓同符号有两套实例缓存
        // ——type_alias_links（get_type_parameter_from_symbol 构造时解析约束）
        // 与 declared_type_links（此处构造不带约束解析恒 None），TypeMapper
        // 全族按 Arc::ptr_eq 匹配，分叉即恒等坍缩（接口成员实例化的 mark
        // mapper sources 与注解实参实例互异时替换落空，wrappedRecursive-
        // GenericType 的 x.a 呈声明态 B<T>）。miss 时先收编 type_alias_links
        // 的合法实例（TypeParameter 旗标）共享回填，消灭后续新建分叉
        if let Some(t) = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
            && t.flags.contains(TypeFlags::TypeParameter)
        {
            self.declared_type_links
                .get_or_default(symbol)
                .declared_type = Some(Arc::clone(&t));
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
            // 检查期环窗口曾把 error 驻留进 declared_type（resolve_alias_body
            // 的窗口产物）：发射期读到即渲染 any。对齐本仓三处既有 error 不
            // 驻留纪律（resolve_type_alias_reference / try_get_type_alias_
            // declared_type / alias_instantiation_cache）：缓存 error 视同
            // 未解析，清掉走重算；真循环由 push_type_resolution 守卫
            if !crate::checker::utilities::is_type_error(&t) {
                return t;
            }
            self.type_alias_links
                .get_or_default(symbol)
                .declared_type = None;
        }
        if self.is_resolving(Arc::as_ptr(symbol), TypeResolutionProperty::DeclaredType) {
            // 重入引用只断路本引用、不标记栈内帧（同
            // resolve_type_alias_reference_with_node 的口径）
            return self.error_type();
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
            // Go 单例不变量（getDeclaredTypeOfTypeAlias checker.go:25276-25282）：
            // mapper 键 typeParameters 与体类型内嵌 T 恒同一实例。本仓体经
            // get_type_from_type_node 解析，T 引用回退 get_type_parameter_from_symbol
            // （type_alias_links 实例）；键登记须同源，否则别名实例化 mapper
            // 按身份匹配脱靶，`T & {}` 不归约呈 NonNullable<any>
            let mut type_parameters: Vec<Arc<Type>> = Vec::new();
            for node in &symbol.declarations {
                if node_kind_is(
                    node,
                    &[
                        SyntaxKind::InterfaceDeclaration,
                        SyntaxKind::ClassDeclaration,
                        SyntaxKind::ClassExpression,
                    ],
                ) || is_type_alias(node)
                {
                    for parameter in alias_type_parameters(node) {
                        if let Some(parameter_symbol) = self.get_symbol_of_declaration(parameter) {
                            type_parameters.push(self.get_type_parameter_from_symbol(&parameter_symbol));
                        }
                    }
                }
            }
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
            // 体求值落 error（环窗口产物）不驻留声明型缓存：对齐本仓 error
            // 不驻留纪律（读臂同口径），真循环由 push_type_resolution 守卫
            if !crate::checker::utilities::is_type_error(&final_t) {
                self.type_alias_links
                    .get_or_default(symbol)
                    .declared_type = Some(Arc::clone(&final_t));
            }
            return final_t;
        }
        self.error_type()
    }

    pub(crate) fn get_emit_syntax_for_module_specifier_expression(
        &mut self,
        usage: &Arc<Node>,
    ) -> tsox_core::core::compiler_options_kinds::ResolutionMode { ::tsox_core::fntrace::enter("get_emit_syntax_for_module_specifier_expression"); 
        // Go getEmitSyntaxForModuleSpecifierExpression（checker.go:15705-15711）：
        // 非字符串字面量说明符无 emit 语法；字符串字面量走
        // GetEmitSyntaxForUsageLocation（compiler/fileloader.go:1097-1119）
        use tsox_core::core::compiler_options_kinds::ResolutionMode as M;
        if !is_string_literal_like(usage) {
            return M::None;
        }
        let Some(source_file) = self.get_source_file_of_node(usage) else {
            return M::None;
        };
        let parent = usage.parent_rc();
        if is_require_call(&parent, false)
            || (is_external_module_reference(&parent)
                && parent.parent_rc().kind == SyntaxKind::ImportEqualsDeclaration)
        {
            return M::CommonJS;
        }
        let file_emit_mode = self
            .program
            .get_emit_module_format_of_file(&source_file.file_name);
        if is_import_call(&walk_up_parenthesized_expressions(&parent)) {
            if tsox_frontend::ast::mig::m3g_3::should_transform_import_call(
                &source_file.file_name,
                &self.compiler_options,
                file_emit_mode,
            ) {
                return M::CommonJS;
            }
            return M::ESNext;
        }
        self.usage_emit_mode_for_import(&source_file.file_name)
    }
}
