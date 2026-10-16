#![allow(unused_imports)]
#![allow(dead_code)]

pub(crate) use super::m1d::r21k7_defs::*;

use tsox_frontend::ast::{get_class_extends_heritage_element, get_name_of_declaration, is_computed_property_name, is_string_literal};
use crate::checker::utilities_is_optional_symbol::is_late_bound_name;
use tsox_core::core::mig::m3j_3::node_core_modules;
use crate::checker::mapper::prepend_type_mapping;
use super::m2c::r21k2_defs::get_class_like_declaration_of_symbol;

use crate::checker::checker::*;
use crate::checker::types::*;
use crate::checker::types::TypeData;
use crate::checker::utilities_is_optional_symbol::is_type_any;
use tsox_frontend::ast::node_data_generated::is_mapped_type_node;
use std::sync::Arc;
use tsox_frontend::ast::{Node, NodeData, Symbol, SyntaxKind};

pub(crate) struct M1dKeyBuilder {
    hasher: std::collections::hash_map::DefaultHasher,
}

impl M1dKeyBuilder {
    pub(crate) fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            hasher: std::collections::hash_map::DefaultHasher::new(),
        }
    }
    pub(crate) fn write_byte(&mut self, b: u8) { ::tsox_core::fntrace::enter("write_byte"); 
        std::hash::Hasher::write_u8(&mut self.hasher, b);
    }
    pub(crate) fn write_type(&mut self, t: &Arc<Type>) { ::tsox_core::fntrace::enter("write_type"); 
        std::hash::Hasher::write_u32(&mut self.hasher, t.id);
    }
    pub(crate) fn write_types(&mut self, types: &[Arc<Type>]) { ::tsox_core::fntrace::enter("write_types"); 
        std::hash::Hasher::write_u32(&mut self.hasher, types.len() as u32);
        for t in types {
            self.write_type(t);
        }
    }
    pub(crate) fn write_alias(&mut self, alias: &Option<Box<TypeAlias>>) { ::tsox_core::fntrace::enter("write_alias"); 
        if let Some(alias) = alias {
            if let Some(symbol) = &alias.symbol {
                std::hash::Hasher::write_u64(&mut self.hasher, symbol.id());
            } else {
                std::hash::Hasher::write_u8(&mut self.hasher, 0);
            }
            self.write_types(&alias.type_arguments);
        } else {
            std::hash::Hasher::write_u8(&mut self.hasher, 0);
        }
    }
    pub(crate) fn hash(self) -> u64 { ::tsox_core::fntrace::enter("hash"); 
        std::hash::Hasher::finish(&self.hasher)
    }
}

pub(crate) fn get_alias_key(alias: &Option<Box<TypeAlias>>) -> u64 { ::tsox_core::fntrace::enter("get_alias_key"); 
    let mut b = M1dKeyBuilder::new();
    b.write_alias(alias);
    b.hash()
}

pub(crate) fn get_conditional_type_key(
    type_arguments: &[Arc<Type>],
    alias: &Option<Box<TypeAlias>>,
    for_constraint: bool,
) -> u64 { ::tsox_core::fntrace::enter("get_conditional_type_key"); 
    let mut b = M1dKeyBuilder::new();
    b.write_types(type_arguments);
    b.write_alias(alias);
    if for_constraint {
        b.write_byte(b'!');
    }
    b.hash()
}

pub(crate) fn get_adjusted_node_for_error(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_adjusted_node_for_error"); 
    if let Some(name) = get_name_of_declaration(node) {
        return name;
    }
    Arc::clone(node)
}

pub(crate) fn get_base_type_node_of_class(t: &Arc<Type>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_base_type_node_of_class"); 
    if let Some(symbol) = &t.symbol {
        let decl = get_class_like_declaration_of_symbol(symbol);
        if let Some(decl) = decl {
            return get_class_extends_heritage_element(&decl);
        }
    }
    None
}

impl Checker {
    pub(crate) fn get_constraint_or_unknown_from_type_parameter(
        &mut self,
        t: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_constraint_or_unknown_from_type_parameter");
        self.get_constraint_from_type_parameter(t)
            .unwrap_or_else(|| self.unknown_type())
    }

    pub(crate) fn get_effective_type_argument_at_index(
        &mut self,
        node: &Arc<Node>,
        type_parameters: &[Arc<Type>],
        index: usize,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_effective_type_argument_at_index"); 
        if let Some(args) = node.type_arguments() {
            if index < args.nodes.len() {
                return self.get_type_from_type_node(&args.nodes[index]);
            }
        }
        let effective = self.get_effective_type_arguments(node, type_parameters);
        Arc::clone(&effective[index])
    }

    pub(crate) fn get_constraint_from_indexed_access(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_constraint_from_indexed_access"); 
        let d = t.indexed_access_data()?;
        let object_type = d.object_type.as_ref()?;
        let index_type = d.index_type.as_ref()?;
        if self.is_mapped_type_generic_indexed_access(t) {
            return Some(self.substitute_indexed_mapped_type(object_type, index_type));
        }
        let index_constraint = self.get_simplified_type_or_constraint(index_type);
        if let Some(constraint) = index_constraint {
            if constraint.id != index_type.id {
                let indexed_access = self.get_indexed_access_type_or_undefined(
                    object_type,
                    &constraint,
                    d.access_flags,
                    None,
                    None,
                );
                if let Some(indexed_access) = indexed_access {
                    return Some(indexed_access);
                }
            }
        }
        let object_constraint = self.get_simplified_type_or_constraint(object_type);
        if let Some(constraint) = object_constraint {
            if constraint.id != object_type.id {
                return self.get_indexed_access_type_or_undefined(
                    &constraint,
                    index_type,
                    d.access_flags,
                    None,
                    None,
                );
            }
        }
        None
    }

    pub(crate) fn get_constraint_from_conditional_type(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_constraint_from_conditional_type"); 
        if let Some(constraint) = self.get_constraint_of_distributive_conditional_type(t) {
            return Some(constraint);
        }
        self.get_default_constraint_of_conditional_type(t)
    }

    pub(crate) fn get_constraint_of_distributive_conditional_type(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_constraint_of_distributive_conditional_type"); 
        if let Some(cached) = t.resolved_constraint_of_distributive() {
            if cached.id != self.no_constraint_type().id {
                return Some(cached);
            }
            return None;
        }
        let d = t.conditional_data()?;
        let root = d.root.as_ref()?;
        let check_type = d.check_type.as_ref()?;
        let mut resolved: Option<Arc<Type>> = None;
        if root.is_distributive && !self.is_restrictive_instantiation(t) {
            let mut constraint = self.get_simplified_type(check_type, false);
            if constraint.id == check_type.id {
                if let Some(c) = self.get_constraint_of_type(&constraint) {
                    constraint = c;
                }
            }
            if constraint.id != check_type.id {
                let mapper = Arc::new(prepend_type_mapping(
                    Arc::clone(check_type),
                    Arc::clone(&constraint),
                    d.mapper.as_deref(),
                ));
                let instantiated =
                    self.get_conditional_type_instantiation(t, Some(mapper.as_ref()), true, None);
                if !instantiated.flags.contains(TypeFlags::Never) {
                    resolved = Some(Arc::clone(&instantiated));
                }
            }
        }
        t.set_resolved_constraint_of_distributive(resolved.clone());
        resolved
    }

    pub(crate) fn get_base_type_variable_of_class(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_base_type_variable_of_class");
        if let Some(class_node) = get_class_like_declaration_of_symbol(symbol) {
            return self.get_base_type_variable_of_class_node(&class_node);
        }
        let class_type = self.get_declared_type_of_class_or_interface(symbol);
        let base_constructor_type = self.get_base_constructor_type_of_class(&class_type)?;
        if base_constructor_type.flags.contains(TYPE_FLAGS_TYPE_VARIABLE) {
            return Some(base_constructor_type);
        }
        if base_constructor_type.flags.contains(TypeFlags::Intersection) {
            return base_constructor_type
                .types()
                .and_then(|ts| {
                    ts.iter()
                        .find(|t| t.flags.contains(TYPE_FLAGS_TYPE_VARIABLE))
                })
                .cloned();
        }
        None
    }

    /// Go getBaseTypeVariableOfClass（checker.go:17922-17942）的节点直算版：
    /// extends 表达式为实体名时先按符号→注解→类型参数直读（Go checkExpression
    /// 对 `Base: C` 形参的值型即类型参数 C 的声明型，getTypeOfSymbol→
    /// getTypeForVariableLikeDeclaration→tryGetTypeFromTypeNode→
    /// getTypeFromTypeReference TypeParameter 臂 checker.go:24556-24558 恒产
    /// TypeFlags.TypeVariable 型）——本仓 get_type_from_type_node 的类型参数
    /// 解析吃实例化帧（type_argument_stack/name_frames 命中即返回映射值），
    /// 建型/显示窗口内帧残留把 C 解析成约束构造型对象（.traces/fn 首调序证
    /// 据：get_base_constructor_type_of_class 的 isConstructorType 经
    /// get_signatures_of_type 拿到非空构签名早退，约束解析函数首调均晚于
    /// set_resolved_base_constructor_type），致交集包裹与显示按名门双双失灵；
    /// 注解直读经 binder 局部链（resolve_name，上下文无关）取类型参数符号后
    /// 走 get_type_parameter_from_symbol 规范型（constraint 驻留），不触帧。
    /// 非注解形态回落实例侧同轮记忆化读视图（base_ctor_type_cache，m2b_6.rs
    /// 是 Object 数据类实例型唯一生效记忆化）与 extends 表达式直算
    /// （check_heritage_value_expression，isConstructorType 门同 Go 的
    /// errorType 转换语义 checker.go:17995-18001）。环安全由消费点的
    /// class_type_resolution_stack 守卫承担（建型在途重入取 any，heritage 表达
    /// 式不自指建型）
    pub(crate) fn get_base_type_variable_of_class_node(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_base_type_variable_of_class_node");
        if let Some(base_type_variable) = self.base_type_variable_of_annotated_heritage(node) {
            return Some(base_type_variable);
        }
        if let Some(base_constructor_type) = self.memoized_base_constructor_type_of_class_node(node)
        {
            return Self::type_variable_of_base_constructor(base_constructor_type);
        }
        let base_type_node = get_class_extends_heritage_element(node)?;
        let expression = base_type_node.expression()?;
        let base_constructor_type = self.check_heritage_value_expression(&expression);
        self.materialize_constraint_from_type_parameter(&base_constructor_type);
        if !self.is_constructor_type(&base_constructor_type) {
            return None;
        }
        Self::type_variable_of_base_constructor(base_constructor_type)
    }

    /// `extends Base`（Base 为带类型参数注解的形参/变量/属性）的基类型变量
    /// 直读：Base 符号优先取检查期已驻留的 symbol_node_links（get_resolved_
    /// symbol 纯缓存读），名字解析回落经容器链取形参符号；类型参数符号层级
    /// 获取——语法位匹配（enclosing）结果须过 TypeParameter 旗，伪命中回落
    /// 名字解析（注解位、heritage 位双位置，同容器链可达箭头 locals），任一
    /// 成功即用；类型参数取 get_type_parameter_from_symbol 规范型，判定段
    /// 同 Go getBaseTypeVariableOfClass（checker.go:17923-17940）按
    /// TypeFlags.TypeVariable 旗判
    fn base_type_variable_of_annotated_heritage(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("base_type_variable_of_annotated_heritage");
        let base_type_node = get_class_extends_heritage_element(node)?;
        let expression = base_type_node.expression()?;
        if expression.kind != SyntaxKind::Identifier {
            return None;
        }
        let base_symbol = self.get_resolved_symbol(&expression).or_else(|| {
            self.resolve_name(
                &expression.text(),
                &expression,
                tsox_frontend::ast::SymbolFlags::VALUE,
                false,
            )
            .map(|s| self.get_merged_symbol(&s))
        })?;
        if self
            .unknown_symbol
            .as_ref()
            .is_some_and(|u| Arc::ptr_eq(u, &base_symbol))
        {
            return None;
        }
        let type_node = match &base_symbol.value_declaration.as_ref()?.data {
            NodeData::ParameterDeclaration(pd) => pd.type_node.as_ref()?,
            NodeData::VariableDeclaration(vd) => vd.type_node.as_ref()?,
            NodeData::PropertyDeclaration(pd) => pd.type_node.as_ref()?,
            _ => return None,
        };
        let NodeData::TypeReferenceNode(tr) = &type_node.data else {
            return None;
        };
        let ref_text = tr.type_name.text();
        let type_meaning = tsox_frontend::ast::SymbolFlags::TYPE;
        let is_type_parameter_decl =
            |s: &Arc<Symbol>| s.declarations.iter().any(|d| matches!(&d.data, NodeData::TypeParameterDeclaration(_)));
        let tp_symbol = self
            .enclosing_type_parameter_symbol(node, &ref_text)
            .filter(is_type_parameter_decl)
            .or_else(|| {
                self.resolve_name(&ref_text, &tr.type_name, type_meaning, false)
                    .map(|s| self.get_merged_symbol(&s))
            })
            .or_else(|| {
                self.resolve_name(&ref_text, &expression, type_meaning, false)
                    .map(|s| self.get_merged_symbol(&s))
            })
            .filter(is_type_parameter_decl)?;
        // Go getBaseTypeVariableOfClass（checker.go:17923-17940）判定段对取得
        // 的基构造型按 TypeFlags.TypeVariable 旗判，无 SymbolFlags 整臂门——
        // 符号门按声明形态（TypeParameterDeclaration 在 declarations）判定，
        // 对旗位漂移免疫；语法位匹配（enclosing）伪命中时按层级回落到名字
        // 解析（注解位与 heritage 位同容器链可达箭头 locals），isConstructorType
        // 门已剥离（checker.go:17995-18001 属检查期 TS2507 流程）
        let declared = self.get_type_parameter_from_symbol(&tp_symbol);
        let base_type_variable = Self::type_variable_of_base_constructor(declared);
        if base_type_variable.is_some() {
            ::tsox_core::fntrace::enter("base_type_variable_of_annotated_heritage_hit");
        }
        base_type_variable
    }

    /// 注解引用名 → 类型参数符号：自类节点沿父链逐层函数型的
    /// type_parameters 声明名匹配（嵌套函数型逐层外扩即名字作用域序），命中
    /// 经 binder symbol_map 直取符号；类自身类型参数不参与（Go
    /// baseExpressionTypeParameters：基类表达式不可引用本类类型参数）
    fn enclosing_type_parameter_symbol(&self, class_node: &Arc<Node>, name: &str) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("enclosing_type_parameter_symbol");
        let mut current = class_node.parent();
        while let Some(ancestor) = current {
            if let Some(tps) =
                crate::checker::grammarchecks_typeparameters::node_type_parameters(&ancestor)
            {
                for tp in tps.nodes.iter() {
                    let matches = match &tp.data {
                        NodeData::TypeParameterDeclaration(tpd) => tpd.name.text() == name,
                        _ => false,
                    };
                    if matches
                        && let Some(sym) = self.program.symbol_map().symbol_of(tp)
                    {
                        ::tsox_core::fntrace::enter("enclosing_type_parameter_symbol_hit");
                        return Some(Arc::clone(sym));
                    }
                }
            }
            current = ancestor.parent();
        }
        None
    }

    /// getBaseConstructorTypeOfClass 的 memoized 读视图：Go 该记忆化驻留在
    /// resolvedBaseConstructorType（checker.go:17950-17956），本仓对 Object 数据
    /// 类实例型实际落 base_ctor_type_cache（m2b_6.rs），只读不触发重算
    fn memoized_base_constructor_type_of_class_node(&self, node: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("memoized_base_constructor_type_of_class_node");
        let own_type = self.class_instance_type_cache.get(&node.id())?;
        own_type
            .as_interface_type()
            .and_then(|i| i.resolved_base_constructor_type.get().cloned())
            .or_else(|| self.base_ctor_type_cache.get(&own_type.id).cloned())
    }

    /// Go getBaseTypeVariableOfClass 判定段（checker.go:17923-17940）：基构造
    /// 类型变量直取，交集取类型变量成分，其余 None
    fn type_variable_of_base_constructor(base_constructor_type: Arc<Type>) -> Option<Arc<Type>> {
        if base_constructor_type.flags.contains(TYPE_FLAGS_TYPE_VARIABLE) {
            return Some(base_constructor_type);
        }
        if base_constructor_type.flags.contains(TypeFlags::Intersection) {
            return base_constructor_type
                .types()
                .and_then(|ts| {
                    ts.iter()
                        .find(|t| t.flags.contains(TYPE_FLAGS_TYPE_VARIABLE))
                })
                .cloned();
        }
        None
    }

    /// Go isConstructorType 型参数臂（checker.go:18007-18009）与
    /// getSignaturesOfType 的型参数委托（getReducedApparentType，
    /// checker.go:20079-20081）都以 getConstraintFromTypeParameter
    /// （checker.go:18066-18103）惰性推导并 memo 进 tp.constraint
    /// （checker.go:18098-18100）为前提；本侧两路读侧均为缓存读，
    /// 签名建型期未驻留 constraint 的声明型参数在类基构造型判定位
    /// 恒 None → 基构造型变量 None → 匿名类值型缺 & T 交集
    ///（emitClassExpressionInDeclarationFile m0 全量 trace 行为证据）。
    /// 本函数仅在该判定位前置物化，恢复 Go 在此点的推导时序，
    /// 不翻转全仓约束读侧语义（惰性化全仓面已验证引入 83 例回归回退）
    pub(crate) fn materialize_constraint_from_type_parameter(
        &mut self,
        t: &Arc<Type>,
    ) { ::tsox_core::fntrace::enter("materialize_constraint_from_type_parameter");
        let Some(tp) = t.as_type_parameter() else {
            return;
        };
        if tp.constraint.is_some() {
            return;
        }
        let target = tp.target.clone();
        let mapper = tp.mapper.clone();
        let derived: Option<Arc<Type>> = if let Some(target) = target {
            self.get_constraint_of_type_parameter(&target)
                .map(|base| self.instantiate_type(&base, mapper.as_ref()))
        } else if let Some(declaration) = self.get_constraint_declaration(t) {
            let mut constraint = self.get_type_from_type_node(&declaration);
            if is_type_any(&constraint) && !self.is_error_type(&constraint) {
                constraint = if declaration
                    .parent()
                    .and_then(|p| p.parent())
                    .as_ref()
                    .is_some_and(|gp| is_mapped_type_node(gp))
                {
                    self.string_number_symbol_type()
                } else {
                    self.unknown_type()
                };
            }
            Some(constraint)
        } else {
            self.get_inferred_type_parameter_constraint(t, false)
        };
        let memo = derived.unwrap_or_else(|| self.no_constraint_type());
        let type_ptr = Arc::as_ptr(t) as *mut Type;
        if let TypeData::TypeParameter(tp) = unsafe { &mut (*type_ptr).data } {
            tp.constraint = Some(memo);
        }
    }

    pub(crate) fn get_declared_type_of_class_or_interface(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_declared_type_of_class_or_interface"); 
        let links = self.declared_type_links.get(symbol);
        if let Some(links) = links {
            if let Some(t) = &links.declared_type {
                return Arc::clone(t);
            }
        }
        self.create_declared_type_of_class_or_interface(symbol)
    }

    pub(crate) fn get_declaration_node_flags_from_symbol(
        &mut self,
        s: &Arc<Symbol>,
    ) -> tsox_frontend::ast::NodeFlags { ::tsox_core::fntrace::enter("get_declaration_node_flags_from_symbol"); 
        if let Some(value_declaration) = &s.value_declaration {
            return self.get_combined_node_flags_cached(value_declaration);
        }
        tsox_frontend::ast::NodeFlags::empty()
    }

    pub(crate) fn get_combined_node_flags_cached(
        &mut self,
        node: &Arc<Node>,
    ) -> tsox_frontend::ast::NodeFlags { ::tsox_core::fntrace::enter("get_combined_node_flags_cached"); 
        self.get_combined_node_flags(node)
    }

    pub(crate) fn get_combined_modifier_flags_cached(
        &mut self,
        node: &Arc<Node>,
    ) -> tsox_frontend::ast::ModifierFlags { ::tsox_core::fntrace::enter("get_combined_modifier_flags_cached"); 
        if self
            .last_combined_modifier_flags_node
            .as_ref()
            .is_some_and(|n| n.id() == node.id())
        {
            return self.last_combined_modifier_flags_result;
        }
        let result = self.get_combined_modifier_flags(node);
        self.last_combined_modifier_flags_node = Some(Arc::clone(node));
        self.last_combined_modifier_flags_result = result;
        result
    }

    pub(crate) fn get_effective_property_name_for_property_name_node(
        &mut self,
        node: &Arc<Node>,
    ) -> (String, bool) { ::tsox_core::fntrace::enter("get_effective_property_name_for_property_name_node"); 
        let name = get_property_name_for_property_name_node(node);
        if !name.is_empty() || !is_computed_property_name(node) {
            if !name.is_empty() {
                return (name, true);
            }
        }
        if is_computed_property_name(node) {
            let Some(expression) = node.expression().cloned() else {
                return (String::new(), false);
            };
            let t = self.get_type_of_expression(&expression);
            if let Some(name) = self.try_get_name_from_type(&t) {
                return (name, true);
            }
            return (String::new(), false);
        }
        (String::new(), false)
    }

    pub(crate) fn get_applicable_index_info_for_name(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<IndexInfo>> { ::tsox_core::fntrace::enter("get_applicable_index_info_for_name"); 
        if is_late_bound_name(name) {
            return self.get_applicable_index_info(t, &self.es_symbol_type());
        }
        let key_type = self.get_string_literal_type(name);
        self.get_applicable_index_info(t, &key_type)
    }

    pub(crate) fn get_constructors_for_type_arguments(
        &mut self,
        t: &Arc<Type>,
        type_argument_nodes: &[Arc<Node>],
        _location: &Arc<Node>,
    ) -> Vec<Arc<Signature>> { ::tsox_core::fntrace::enter("get_constructors_for_type_arguments");
        let type_arg_count = type_argument_nodes.len();
        self.intersection_mixin_aware_construct_signatures(t)
            .into_iter()
            .filter(|sig| {
                let min = self.get_min_type_argument_count(&sig.type_parameters);
                type_arg_count >= min && type_arg_count <= sig.type_parameters.len()
            })
            .collect()
    }

    pub(crate) fn get_cannot_resolve_module_name_error_for_specific_module(
        &mut self,
        module_name: &Arc<Node>,
    ) -> Option<tsox_core::diagnostics::Message> { ::tsox_core::fntrace::enter("get_cannot_resolve_module_name_error_for_specific_module"); 
        if is_string_literal(module_name) {
            if node_core_modules().get(module_name.text()).copied().unwrap_or(false) {
                if self.compiler_options.uses_wildcard_types() {
                    return Some(tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE);
                }
                return Some(tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE_AND_THEN_ADD_NODE_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG);
            }
        }
        None
    }
}
