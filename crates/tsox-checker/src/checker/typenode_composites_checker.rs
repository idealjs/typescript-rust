#![allow(unused_imports)]

use crate::checker::exports_union_reduction::UnionReduction;
use crate::checker::types::IntersectionFlags;
use crate::checker::typenode_composites::*;

impl Checker {
    pub(crate) fn get_type_from_array_or_tuple_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_array_or_tuple_type_node"); 
        match &node.data {
            NodeData::ArrayTypeNode(d) => {
                let elem_type = self.get_type_from_type_node(&d.element_type);
                let t = self.create_array_type(elem_type);
                self.attach_type_alias_from_node(node, &t)
            }
            NodeData::TupleTypeNode(d) => {
                let mut element_types = Vec::new();

                let mut element_infos = Vec::new();
                let mut variadic_types: Vec<Arc<Type>> = Vec::new();
                let mut has_variadic_union = false;
                for elem in d.elements.iter() {
                    if let NodeData::RestTypeNode(rd) = &elem.data {
                        let inner = Arc::clone(&rd.type_node);
                        let inner_t = self.get_type_from_type_node(&inner);
                        has_variadic_union |= inner_t.flags.contains(TypeFlags::Never)
                            || matches!(&inner_t.data, TypeData::Union(_));
                        variadic_types.push(inner_t);
                    }
                    element_infos.push(self.get_tuple_element_info(elem));
                    element_types.push(self.get_type_from_type_node(elem));
                }
                if has_variadic_union && !self.check_cross_product_union(node, &variadic_types) {
                    return self.error_type();
                }
                let readonly = node.parent().as_ref().is_some_and(|p| {
                    matches!(&p.data, NodeData::TypeOperatorNode(op)
                        if op.operator == SyntaxKind::ReadonlyKeyword)
                });
                let (element_types, element_infos) =
                    self.normalize_variadic_tuple_elements(element_types, &element_infos);
                let t = self.create_tuple_type_ex(element_types, element_infos, readonly);
                self.attach_type_alias_from_node(node, &t)
            }
            _ => self.error_type(),
        }
    }


    pub(crate) fn get_type_from_optional_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_optional_type_node"); 
        let inner = node.type_node().expect("OptionalType has type").clone();
        let t = self.get_type_from_type_node(&inner);
        self.add_optionality(&t)
    }

    pub(crate) fn get_type_from_union_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_union_type_node"); 
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let types = match &node.data {
            NodeData::UnionTypeNode(data) => &data.types,
            _ => return self.error_type(),
        };
        let mut member_types = Vec::new();
        for member in types.iter() {
            member_types.push(self.get_type_from_type_node(member));
        }
        // Go getTypeFromUnionTypeNode：getUnionTypeEx(Literal, alias, nil) 走
        // worker——ContainsIntersections 旗标随 includes 落到产物（getReducedType
        // 的 union 归约分支剔除 never 交集成员依赖它），alias 进驻留键
        let alias = self.alias_symbol_for_type_node(node).map(|sym| {
            let (tp_symbols, _) = self.collect_alias_type_params_and_body(&sym);
            let args = tp_symbols
                .iter()
                .map(|tp| self.get_type_parameter_from_symbol(tp))
                .collect();
            crate::checker::types::TypeAlias::new(Some(sym), args)
        });
        let result = self.get_union_type_worker(
            member_types,
            UnionReduction::Literal,
            alias.as_ref(),
            None,
        );
        self.attach_alias_for_type_node(node, &result);
        // 环窗口内 error 成员坍缩产物（getUnionTypeWorker 的 INCLUDES_ERROR 臂）
        // 不驻留节点缓存：对齐 get_type_from_type_node 的 error 不驻留纪律，
        // error 驻留会永久掩盖窗口外的正确重解析（unionTypeWithIndexedLiteralType
        // 的 >U : error 恒定复现即此形态）
        if !crate::checker::utilities::is_type_error(&result) {
            self.cache_type(node, result.clone());
        }
        result
    }

    pub(crate) fn get_type_from_intersection_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_intersection_type_node"); 
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let types = match &node.data {
            NodeData::IntersectionTypeNode(data) => &data.types,
            _ => return self.error_type(),
        };
        let alias = self.get_alias_for_type_node(node);
        let mut member_types = Vec::new();
        for member in types.iter() {
            member_types.push(self.get_type_from_type_node(member));
        }

        // Go getTypeFromIntersectionTypeNode（checker.go:25657-25666）：X & {} 中
        // X 为 string/number/bigint 或 pattern template literal 时不做 supertype
        // 归约，保留 "a" | "b" | string & {} 形态的字面量
        let mut flags = IntersectionFlags::empty();
        if member_types.len() == 2 {
            if let Some(empty_index) = member_types
                .iter()
                .position(|t| Arc::ptr_eq(t, &self.empty_type_literal_type()))
            {
                let t = &member_types[1 - empty_index];
                if t.flags.intersects(TypeFlags::String | TypeFlags::Number | TypeFlags::BigInt)
                    || t.flags.intersects(TypeFlags::TemplateLiteral) && self.is_pattern_literal_type(t)
                {
                    flags = IntersectionFlags(
                        flags.bits() | IntersectionFlags::NO_SUPERTYPE_REDUCTION.bits(),
                    );
                }
            }
        }
        // Go getTypeFromIntersectionTypeNode（checker.go:25653-25673）：归约走
        // getIntersectionTypeEx 全量臂——含 IncludesEmptyObject 聚合位触发
        // （checker.go:27660-27666）与 removeRedundantSupertypes 的空对象剔除
        // 臂（checker.go:27867 IsEmptyAnonymousObjectType &&
        // DefinitelyNonNullable），`{mapped} & {}` 由之坍缩单成员；简化版
        // get_intersection_type 无空对象臂，坍缩不发生则节点级别名附加后
        // >Id 行回退 Id<T> 形。节点级别名按 Go 传入 ex
        //（checker.go:25656 getAliasForTypeNode → :25672 getIntersectionTypeEx
        // 实参）：归约含并集时经 getUnionTypeEx 带 alias 重构
        //（checker.go:27767-27775），多成分交集落 result.alias
        //（checker.go:27771），单成员坍缩裸返回丢别名（checker.go:27694）
        let alias = self.alias_symbol_for_type_node(node).map(|sym| {
            let args = self.get_type_arguments_for_alias_symbol(Some(&sym));
            crate::checker::types::TypeAlias::new(Some(sym), args)
        });
        let result = self.get_intersection_type_ex(
            &member_types,
            flags,
            alias.as_ref(),
        );
        // 同 get_type_from_union_type_node：环窗口 error 不驻留节点缓存
        if !crate::checker::utilities::is_type_error(&result) {
            self.cache_type(node, result.clone());
        }
        result
    }

    pub(crate) fn get_type_from_named_tuple_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_named_tuple_type_node"); 
        let inner = node.type_node().expect("NamedTupleMember has type").clone();
        self.get_type_from_type_node(&inner)
    }

    pub(crate) fn get_type_from_rest_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_rest_type_node"); 
        let inner = node.type_node().expect("RestType has type").clone();
        match self.array_element_type_node(&inner) {
            Some(element_node) => self.get_type_from_type_node(&element_node),
            None => self.get_type_from_type_node(&inner),
        }
    }

    fn array_element_type_node(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("array_element_type_node"); 
        match &node.data {
            NodeData::ParenthesizedTypeNode(p) => {
                self.array_element_type_node(&p.type_node)
            }
            NodeData::TupleTypeNode(t) => {
                if t.elements.len() != 1 {
                    return None;
                }
                let first = t.elements.iter().next()?;
                match &first.data {
                    NodeData::RestTypeNode(r) => {
                        self.array_element_type_node(&r.type_node)
                    }
                    NodeData::NamedTupleMember(m) if m.dot_dot_dot_token.is_some() => {
                        self.array_element_type_node(&m.type_node)
                    }
                    _ => None,
                }
            }
            NodeData::ArrayTypeNode(a) => Some(Arc::clone(&a.element_type)),
            _ => None,
        }
    }
}
