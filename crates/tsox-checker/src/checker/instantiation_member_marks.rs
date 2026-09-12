#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::types::*;
use std::sync::Arc;

impl Checker {
    /// Go resolveStructuredTypeMembers 的实例化语义（显示侧元数据）：
    /// 泛型容器实例中、声明类型经 mapper 会变化的成员符号标 Instantiated 并挂
    /// mapper，nodebuilder 限定名据此写 `A<string>.m`（lookupInstantiatedTypeArgumentNodes）。
    /// 类型不含被映射类型参数的成员（Go instantiateSymbol 同指针捷径）保持声明符号，
    /// 显示回退到类型参数名形态 `A<T>.m`。
    pub(crate) fn mark_structured_members_instantiated(
        &mut self,
        instance: &Arc<Type>,
        container: &Arc<Symbol>,
        tp_symbols: &[Arc<Symbol>],
        type_params: &[Arc<Type>],
        type_args: &[Arc<Type>],
    ) {
        if type_params.is_empty() || type_params.len() != type_args.len() {
            return;
        }
        let Some(structured) = instance.as_structured() else {
            return;
        };
        let mut member_syms: Vec<Arc<Symbol>> = Vec::new();
        for sym in structured.members.entries.values().chain(structured.properties.iter()) {
            if !member_syms.iter().any(|s| Arc::ptr_eq(s, sym)) {
                member_syms.push(Arc::clone(sym));
            }
        }
        if member_syms.is_empty() {
            return;
        }
        let sources: Vec<Arc<Type>> = type_params.to_vec();
        let targets: Vec<Arc<Type>> = type_args.to_vec();
        let mapper = Arc::new(crate::checker::mapper::new_array_type_mapper(
            sources, targets,
        ));
        for sym in member_syms {
            if !self.member_declaration_touches_type_params(&sym, tp_symbols) {
                continue;
            }
            let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
            unsafe {
                (*sym_mut).check_flags |= CheckFlags::Instantiated;
                if (*sym_mut).parent().is_none() {
                    (*sym_mut).set_parent(container);
                }
            }
            self.value_symbol_links
                .get_or_default(&sym)
                .mapper = Some(Arc::clone(&mapper));
            self.instantiated_member_owner
                .insert(Arc::as_ptr(&sym) as usize, u64::from(instance.id));
        }
    }

    /// 成员声明位（注解/参数）是否引用容器的类型参数符号（Go instantiateSymbol
    /// 的类型变化判定近似：注解不含类型参数时类型经 mapper 不变）
    fn member_declaration_touches_type_params(
        &mut self,
        member: &Arc<Symbol>,
        tp_symbols: &[Arc<Symbol>],
    ) -> bool {
        let touches = |node: &Arc<tsox_frontend::ast::Node>| {
            node_touches_type_param_symbols(node, tp_symbols)
        };
        member.declarations.iter().any(|decl| {
            let type_node = member_type_node(decl);
            // 无注解成员（体推断型）无法静态判定：保守视为触碰（A<string>.foo 形态）
            let ann_touches = match type_node.as_ref() {
                Some(tn) => touches(tn),
                None => true,
            };
            let params_touches = member_parameters(decl).is_some_and(|params| {
                params.iter().any(|p| {
                    let ptn = match &p.data {
                        tsox_frontend::ast::NodeData::ParameterDeclaration(pd) => {
                            pd.type_node.clone()
                        }
                        _ => None,
                    };
                    ptn.as_ref().is_some_and(&touches)
                })
            });
            ann_touches || params_touches
        })
    }

    /// 实例化成员的容器实参：容器声明类型参数经成员 mapper 映射（Go
    /// lookupInstantiatedTypeArgumentNodes 的 params+mapper.map 路径）
    pub(crate) fn instantiated_member_container_args(
        &mut self,
        member: &Arc<Symbol>,
        container: &Arc<Symbol>,
    ) -> Option<Vec<Arc<Type>>> {
        if !member
            .check_flags
            .contains(CheckFlags::Instantiated)
        {
            return None;
        }
        let mapper = self
            .value_symbol_links
            .get(member)
            .and_then(|l| l.mapper.clone())?;
        let params = self.declared_type_parameter_types(container);
        if params.is_empty() {
            return None;
        }
        let args: Vec<Arc<Type>> = params.iter().map(|p| mapper.map(p)).collect();
        if args
            .iter()
            .zip(params.iter())
            .any(|(a, p)| Arc::ptr_eq(a, p))
        {
            return None;
        }
        Some(args)
    }

    /// 类声明的类型参数类型（构造签名与实例化共用，Go LocalTypeParameters）
    pub(crate) fn class_type_parameter_types_of(
        &mut self,
        class_node: &Arc<tsox_frontend::ast::Node>,
    ) -> Vec<Arc<Type>> {
        let tps = match &class_node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
            tsox_frontend::ast::NodeData::ClassExpression(d) => d.type_parameters.as_ref(),
            _ => return Vec::new(),
        };
        let Some(tps) = tps else {
            return Vec::new();
        };
        let tp_syms: Vec<Arc<Symbol>> = {
            let sym_map = self.program.symbol_map();
            tps.iter()
                .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                .collect()
        };
        tp_syms
            .iter()
            .map(|s| self.get_type_parameter_from_symbol(s))
            .collect()
    }
}

fn member_type_node(
    decl: &Arc<tsox_frontend::ast::Node>,
) -> Option<Arc<tsox_frontend::ast::Node>> {
    use tsox_frontend::ast::NodeData;
    match &decl.data {
        NodeData::PropertySignatureDeclaration(d) => Some(Arc::clone(&d.type_node)),
        NodeData::MethodSignatureDeclaration(d) => d.type_node.clone(),
        NodeData::PropertyDeclaration(d) => d.type_node.clone(),
        NodeData::MethodDeclaration(d) => d.type_node.clone(),
        NodeData::GetAccessorDeclaration(d) => d.type_node.clone(),
        NodeData::SetAccessorDeclaration(d) => d.type_node.clone(),
        _ => None,
    }
}

fn member_parameters(
    decl: &Arc<tsox_frontend::ast::Node>,
) -> Option<Arc<tsox_frontend::ast::NodeList>> {
    use tsox_frontend::ast::NodeData;
    match &decl.data {
        NodeData::MethodSignatureDeclaration(d) => Some(Arc::clone(&d.parameters)),
        NodeData::MethodDeclaration(d) => Some(Arc::clone(&d.parameters)),
        NodeData::GetAccessorDeclaration(d) => Some(Arc::clone(&d.parameters)),
        NodeData::SetAccessorDeclaration(d) => Some(Arc::clone(&d.parameters)),
        _ => None,
    }
}

fn node_touches_type_param_symbols(
    node: &Arc<tsox_frontend::ast::Node>,
    tp_symbols: &[Arc<Symbol>],
) -> bool {
    use tsox_frontend::ast::SyntaxKind;
    if node.kind == SyntaxKind::Identifier
        && tp_symbols.iter().any(|s| s.name == node.text())
    {
        return true;
    }
    let mut found = false;
    tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
        if node_touches_type_param_symbols(child, tp_symbols) {
            found = true;
            true
        } else {
            false
        }
    });
    found
}
