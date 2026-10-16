#![allow(unused_imports)]
use super::m1d_2::get_alias_key;
use super::wc1b::is_tuple_type;
use tsox_frontend::ast::is_in_js_file;
use tsox_frontend::ast::node_data_generated::is_jsdoc_augments_tag;
use super::wc3_3::is_local_type_alias;
use crate::checker::nodecopy_property_name::is_numeric_literal_name;
use super::m2a::is_rest_parameter;
use tsox_core::jsnum;
use tsox_frontend::ast::is_string_literal_like;
use tsox_frontend::ast::mig::m3g_2::is_type_reference_type;

use crate::checker::checker::*;
use crate::checker::types_cached_type_kind::CacheHashKey;
use tsox_frontend::ast::{Node, NodeData, Symbol, SymbolFlags, SyntaxKind};

use super::m1b::jsnum_from_string;
use super::m2a::r20k6_defs::R20K6CheckerExt;
use tsox_frontend::ast::node_data_generated::is_import_attributes;

impl Checker {
    pub(crate) fn get_type_from_class_or_interface_reference(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_class_or_interface_reference"); 
        let merged = self.get_merged_symbol(symbol);
        let t = self.get_declared_type_of_class_or_interface(&merged);
        let type_parameters: &[Arc<Type>] = t
            .as_interface_type()
            .map(|d| d.local_type_parameters())
            .unwrap_or(&[]);
        if !type_parameters.is_empty() {
            let num_type_arguments = node.type_arguments().map(|l| l.nodes.len()).unwrap_or(0);
            let min_type_argument_count = self.get_min_type_argument_count(&type_parameters);
            let is_js = is_in_js_file(node);
            let is_js_implicit_any = !self.no_implicit_any && is_js;
            if !is_js_implicit_any
                && (num_type_arguments < min_type_argument_count
                    || num_type_arguments > type_parameters.len())
            {
                let message = if is_js
                    && node.kind == SyntaxKind::ExpressionWithTypeArguments
                    && node
                        .parent()
                        .is_some_and(|p| !is_jsdoc_augments_tag(&p))
                {
                    if min_type_argument_count < type_parameters.len() {
                        tsox_core::diagnostics::messages_generated::EXPECTED_0_1_TYPE_ARGUMENTS_PROVIDE_THESE_WITH_AN_EXTENDS_TAG
                    } else {
                        tsox_core::diagnostics::messages_generated::EXPECTED_0_TYPE_ARGUMENTS_PROVIDE_THESE_WITH_AN_EXTENDS_TAG
                    }
                } else if min_type_argument_count < type_parameters.len() {
                    tsox_core::diagnostics::messages_generated::GENERIC_TYPE_0_REQUIRES_BETWEEN_1_AND_2_TYPE_ARGUMENTS
                } else {
                    tsox_core::diagnostics::messages_generated::GENERIC_TYPE_0_REQUIRES_1_TYPE_ARGUMENT_S
                };
                let type_str =
                    self.type_to_string_ex(&t, crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags::WRITE_ARRAY_AS_GENERIC);
                self.error_message(
                    node,
                    message,
                    &[type_str, min_type_argument_count.to_string(), type_parameters.len().to_string()],
                );
                if !is_js {
                    return self.error_type();
                }
            }
            if node.kind == SyntaxKind::TypeReference
                && self.is_deferred_type_reference_node(
                    node,
                    num_type_arguments != type_parameters.len(),
                )
            {
                return self.create_deferred_type_reference(&t, Some(node), None, None);
            }
            let local_type_arguments = self.get_type_arguments_from_node(node);
            let local_type_arguments = self.fill_missing_type_arguments(
                &local_type_arguments,
                type_parameters,
                min_type_argument_count,
                is_js,
            );
            let mut all_type_arguments: Vec<Arc<Type>> = t
                .as_interface_type()
                .map(|d| d.outer_type_parameters())
                .unwrap_or(&[])
                .to_vec();
            all_type_arguments.extend(local_type_arguments);
            return self.create_type_reference_ex(&t, &all_type_arguments, ObjectFlags::FromTypeNode);
        }
        if self.check_no_type_arguments(node, Some(symbol)) {
            return t;
        }
        self.error_type()
    }

    pub(crate) fn get_type_from_import_attributes(
        &mut self,
        node: Option<&Arc<Node>>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_from_import_attributes"); 
        let node = node?;
        if is_import_attributes(node) {
            return Some(self.check_import_attributes_expression(node));
        }
        Some(self.check_expression_cached(node))
    }

    pub(crate) fn get_type_from_index_infos_of_contextual_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
        name_type: Option<&Arc<Type>>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_from_index_infos_of_contextual_type"); 
        if is_tuple_type(t) && is_numeric_literal_name(name) && jsnum_from_string(name).0 >= 0.0 {
            if let Some(target) = t.target_tuple_type() {
                let rest_type = self.get_element_type_of_slice_of_tuple_type(
                    t,
                    target.fixed_length as i32,
                    0,
                    false,
                    true,
                );
                if rest_type.is_some() {
                    return rest_type;
                }
            }
        }
        let name_type = match name_type {
            Some(nt) => Arc::clone(nt),
            None => self.get_string_literal_type(name),
        };
        let mut index_infos = self.get_index_infos_of_structured_type(t);
        if index_infos.is_empty()
            && let Some(sym) = t.symbol.as_ref()
            && sym.flags.contains(SymbolFlags::Interface)
        {
            // Go getIndexInfosOfType（checker.go:20096）→
            // getIndexInfosOfStructuredType（checker.go:19297-19300）：结构化成员
            // 未解析时惰性解析；本侧壳/声明型缓存双对象下 structured 空读漏掉
            // 接口索引信息，改读符号声明型缓存（解析窗内该符号照旧不补，
            // 维持环窗口纪律）
            let sym_ptr = Arc::as_ptr(sym) as *const Symbol;
            if !self.is_resolving(
                sym_ptr,
                crate::checker::checker::TypeResolutionProperty::DeclaredType,
            ) {
                let declared = self.resolve_interface_type_ex(sym, None);
                if !crate::checker::utilities::is_type_error(&declared)
                    && let Some(infos) = declared.as_structured().map(|s| s.index_infos.clone())
                {
                    index_infos = infos;
                }
            }
        }
        let index_info = self.find_applicable_index_info(&index_infos, &name_type)?;
        self.transient_index_value_of(index_info.as_ref())
    }

    /// 索引值型读臂：解析窗内别名重入断路落下的 error 按出生点记账换回别名
    /// 声明型（Go getTypeFromIndexInfosOfContextualType（checker.go:32412-32426）
    /// 返回 indexInfo.valueType——惰性解析使该值在窗外取得时别名已驻留声明型，
    /// 本侧急切求值窗内的断路 error 以记账表还原同一答案）
    pub(crate) fn transient_index_value_of(
        &mut self,
        index_info: &crate::checker::IndexInfo,
    ) -> Option<Arc<Type>> {
        let value = index_info.value_type.clone();
        if !value.as_ref().is_some_and(|v| crate::checker::utilities::is_type_error(v)) {
            return value;
        }
        let alias = index_info
            .declaration
            .as_ref()
            .and_then(|d| self.transient_alias_index_values.get(&(d.id() as usize)).cloned());
        match alias {
            Some(sym) => Some(self.get_declared_type_of_type_alias(&sym)),
            None => value,
        }
    }

    pub(crate) fn get_type_from_property_descriptor(
        &mut self,
        node: &Arc<Node>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_property_descriptor"); 
        let object_literal_type = self.check_expression_cached(node);
        if let Some(value_type) = self.get_type_of_property_of_type(&object_literal_type, "value") {
            return value_type;
        }
        if let Some(get_func) = self.get_type_of_property_of_type(&object_literal_type, "get") {
            if let Some(get_sig) = self.get_single_call_signature(&get_func) {
                return self
                    .get_return_type_of_signature(&get_sig)
                    .unwrap_or_else(|| self.unknown_type());
            }
        }
        if let Some(set_func) = self.get_type_of_property_of_type(&object_literal_type, "set") {
            if let Some(set_sig) = self.get_single_call_signature(&set_func) {
                {
                    return self.get_type_of_first_parameter_of_signature(&set_sig);
                }
            }
        }
        self.any_type()
    }

    pub(crate) fn get_type_from_type_alias_reference(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_type_alias_reference");
        // Go checker.go:24968 node.TypeArguments()（ast.go:530-545 含
        // TypeReferenceNode）；Node::type_arguments（m4o_3.rs:571）缺该 kind
        let type_arguments = tsox_frontend::ast::mig::m3c::type_arguments(node);
        if symbol
            .check_flags
            .contains(CheckFlags::Unresolved)
        {
            let alias = TypeAlias {
                symbol: Some(Arc::clone(symbol)),
                type_arguments: type_arguments
                    .iter()
                    .map(|n| self.get_type_from_type_node(n))
                    .collect(),
            };
            let key = CacheHashKey::new(get_alias_key(&Some(Box::new(alias))), 0);
            if let Some(error_type) = self.error_types.get(&key) {
                return Arc::clone(error_type);
            }
            let mut error_type = self.new_intrinsic_type(TypeFlags::ANY, "error");
            let alias = TypeAlias {
                symbol: Some(Arc::clone(symbol)),
                type_arguments: type_arguments
                    .iter()
                    .map(|n| self.get_type_from_type_node(n))
                    .collect(),
            };
            if let Some(et) = Arc::get_mut(&mut error_type) {
                et.alias = Some(Box::new(alias));
            }
            self.error_types.insert(key, Arc::clone(&error_type));
            return error_type;
        }
        let t = self.get_declared_type_of_symbol(symbol);
        let type_parameters = self
            .type_alias_links
            .get(symbol)
            .map(|l| l.type_parameters.clone())
            .unwrap_or_default();
        if !type_parameters.is_empty() {
            let num_type_arguments = type_arguments.len();
            let min_type_argument_count = self.get_min_type_argument_count(&type_parameters);
            if num_type_arguments < min_type_argument_count
                || num_type_arguments > type_parameters.len()
            {
                let message = if min_type_argument_count == type_parameters.len() {
                    tsox_core::diagnostics::messages_generated::GENERIC_TYPE_0_REQUIRES_1_TYPE_ARGUMENT_S
                } else {
                    tsox_core::diagnostics::messages_generated::GENERIC_TYPE_0_REQUIRES_BETWEEN_1_AND_2_TYPE_ARGUMENTS
                };
                let symbol_str = self.symbol_to_string(symbol);
                self.error_message(
                    node,
                    message,
                    &[symbol_str, min_type_argument_count.to_string(), type_parameters.len().to_string()],
                );
                return self.error_type();
            }
            let alias_symbol = self.get_alias_symbol_for_type_node(node);
            let mut new_alias_symbol: Option<Arc<Symbol>> = None;
            if let Some(alias_symbol) = alias_symbol.clone() {
                if is_local_type_alias(symbol) || !is_local_type_alias(&alias_symbol) {
                    new_alias_symbol = Some(alias_symbol);
                }
            }
            let mut alias_type_arguments: Vec<Arc<Type>> = Vec::new();
            if new_alias_symbol.is_none() {
                new_alias_symbol = None;
            }
            if let Some(new_alias_symbol) = &new_alias_symbol {
                alias_type_arguments = self.get_type_arguments_for_alias_symbol(Some(new_alias_symbol));
            } else if is_type_reference_type(node) {
                let resolved_alias_symbol =
                    self.resolve_type_reference_name(node, SymbolFlags::Alias, true);
                if let Some(resolved_alias_symbol) = resolved_alias_symbol {
                    if !Arc::ptr_eq(&resolved_alias_symbol, &self.unknown_symbol()) {
                        {
                            let resolved = self.resolve_alias(&resolved_alias_symbol);
                            if resolved.flags.contains(SymbolFlags::TypeAlias) {
                                new_alias_symbol = Some(Arc::clone(&resolved));
                                alias_type_arguments = self.get_type_arguments_from_node(node);
                            }
                        }
                    }
                }
                // 引用目标即本 TypeAlias 时以引用实参补挂 newAlias：实例化产物
                // 携带 TypeAlias{symbol, args}，呈现端别名门控（可及为真）呈
                // `Id<…>` 形。Go 实例化呈别名形为常态（Record<…>、
                // declarationEmitInferredTypeAlias8 `>x : Foo<number[]>`），
                // 声明名位别名仅交集单成分归约丢弃（checker.go:27680-27682）。
                // 现径两条来源（getAliasSymbolForTypeNode 声明宿主臂、
                // Alias-meaning 导入别名臂）对普通类型别名引用均不命中：
                // getAliasSymbolForTypeNode 宿主走到的非声明节点（checker.go:
                // 25113-25123）、resolveTypeReferenceName 的 Alias meaning 经
                // getSymbol 位过滤（checker.go:2272-2288）拒 TypeAlias 位符号
                if new_alias_symbol.is_none() && symbol.flags.contains(SymbolFlags::TypeAlias) {
                    new_alias_symbol = Some(Arc::clone(symbol));
                    alias_type_arguments = self.get_type_arguments_from_node(node);
                }
            }
            let new_alias = new_alias_symbol.as_ref().map(|s| {
                Arc::new(TypeAlias {
                    symbol: Some(Arc::clone(s)),
                    type_arguments: alias_type_arguments.clone(),
                })
            });
            let args = self.get_type_arguments_from_node(node);
            return self.get_type_alias_instantiation(symbol, &args, new_alias.as_ref());
        }
        if self.check_no_type_arguments(node, Some(symbol)) {
            return t;
        }
        self.error_type()
    }
}

pub(crate) fn has_type_json_import_attribute(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_type_json_import_attribute"); 
    let NodeData::ImportDeclaration(d) = &node.data else {
        return false;
    };
    let Some(attributes) = &d.attributes else {
        return false;
    };
    let NodeData::ImportAttributes(attrs) = &attributes.data else {
        return false;
    };
    attrs
        .attributes
        .nodes
        .iter()
        .any(|attr| {
            let NodeData::ImportAttribute(a) = &attr.data else {
                return false;
            };
            attr.name().as_deref().map(|n| n.text()) == Some("type")
                && is_string_literal_like(&a.value)
                && a.value.text() == "json"
        })
}

pub(crate) fn has_type_parameter_by_name(type_parameters: &[Arc<Type>], name: &str) -> bool { ::tsox_core::fntrace::enter("has_type_parameter_by_name"); 
    type_parameters.iter().any(|tp| {
        tp.symbol
            .as_ref()
            .is_some_and(|s| s.name == name)
    })
}

pub(crate) fn get_unique_type_parameter_name(type_parameters: &[Arc<Type>], base_name: &str) -> String { ::tsox_core::fntrace::enter("get_unique_type_parameter_name"); 
    let mut base_name = base_name.to_string();
    while base_name.len() > 1
        && base_name
            .chars()
            .last()
            .is_some_and(|c| c.is_ascii_digit())
    {
        base_name.pop();
    }
    let mut index = 1;
    loop {
        let augmented_name = format!("{}{}", base_name, index);
        if !has_type_parameter_by_name(type_parameters, &augmented_name) {
            return augmented_name;
        }
        index += 1;
    }
}

pub(crate) fn has_common_dom_type_name(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("has_common_dom_type_name"); 
    let Some(symbol) = &t.symbol else {
        return false;
    };
    let name = &symbol.name;
    name == "EventTarget"
        || name == "Node"
        || name == "Element"
        || (name.starts_with("HTML") && name.ends_with("Element"))
}

pub(crate) fn has_rest_parameter(signature: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_rest_parameter"); 
    signature
        .parameters()
        .and_then(|l| l.nodes.last())
        .is_some_and(|last| is_rest_parameter(last))
}
