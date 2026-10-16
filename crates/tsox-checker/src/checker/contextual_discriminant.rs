#![allow(unused_imports)]

use std::collections::HashSet;
use std::sync::Arc;

use tsox_frontend::ast::{CheckFlags, Node, NodeData, SyntaxKind};

use crate::checker::relater_relate_impl_chunk::*;

const TRUE: i8 = 1;
const FALSE: i8 = 0;
const MAYBE: i8 = -1;

fn type_flags_primitive() -> TypeFlags { ::tsox_core::fntrace::enter("type_flags_primitive"); 
    TypeFlags::from_bits_truncate(
        crate::checker::types::TYPE_FLAGS_STRING_LIKE.bits()
            | crate::checker::types::TYPE_FLAGS_NUMBER_LIKE.bits()
            | crate::checker::types::TYPE_FLAGS_BIG_INT_LIKE.bits()
            | crate::checker::types::TYPE_FLAGS_BOOLEAN_LIKE.bits()
            | crate::checker::types::TYPE_FLAGS_ENUM_LIKE.bits()
            | TypeFlags::ESSymbol.bits()
            | TypeFlags::UniqueESSymbol.bits()
            | crate::checker::types::TYPE_FLAGS_VOID_LIKE.bits()
            | TypeFlags::Null.bits(),
    )
}

fn distributed(t: &Arc<Type>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("distributed"); 
    match &t.data {
        TypeData::Union(u) => u.union_or_intersection.types.to_vec(),
        _ => vec![Arc::clone(t)],
    }
}

enum DiscriminantItem {
    PropertyAssignment(Arc<Node>),
    ShorthandProperty(Arc<Node>),
    MissingMember,
}

impl DiscriminantItem {
    fn name(&self, checker: &Checker) -> Option<String> { ::tsox_core::fntrace::enter("name"); 
        let node = match self {
            DiscriminantItem::PropertyAssignment(n) | DiscriminantItem::ShorthandProperty(n) => n,
            DiscriminantItem::MissingMember => return None,
        };
        checker.node_property_name(node)
    }
}

impl Checker {
    fn node_property_name(&self, node: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("node_property_name"); 
        let name = match &node.data {
            NodeData::PropertyAssignment(d) => &d.name,
            NodeData::ShorthandPropertyAssignment(d) => &d.name,
            _ => return None,
        };
        match &name.data {
            NodeData::Identifier(id) => Some(id.text.clone()),
            NodeData::StringLiteral(s) => Some(s.text.clone()),
            NodeData::NumericLiteral(n) => Some(n.text.clone()),
            _ => None,
        }
    }

    /// Go isDiscriminantProperty（relater.go:1078）：联合的合成属性上
    /// NonUniform+Literal 且属性型非泛型即为判别式属性（IsDiscriminant
    /// 计算缓存省略，符号每查询重建）
    pub(crate) fn is_discriminant_property(&mut self, t: &Arc<Type>, name: &str) -> bool { ::tsox_core::fntrace::enter("is_discriminant_property"); 
        if !t.is_union() {
            return false;
        }
        let Some(prop) = self.get_union_or_intersection_property(t, name) else {
            return false;
        };
        if !prop.check_flags.contains(CheckFlags::SyntheticProperty) {
            return false;
        }
        let prop_type = self.get_type_of_symbol(&prop);
        prop.check_flags.contains(
            CheckFlags::HasNonUniformType.union(CheckFlags::HasLiteralType),
        ) && !self.is_generic_type(&prop_type)
    }

    /// Go discriminateContextualTypeByObjectMembers（checker.go:31099）：
    /// getMatchingUnionConstituentForObjectLiteral 依赖 >=10 成员的
    /// keyPropertyName 索引，小联合下恒为 None，未移植；
    /// discriminatedContextualTypes memo 只驻留成功判别（Go nil 写入
    /// 为 no-op），重入递归断路由 discriminant_item_matches 的上下文
    /// 无关取型承担
    pub(crate) fn discriminate_contextual_type_by_object_members(
        &mut self,
        node: &Arc<Node>,
        contextual_type: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("discriminate_contextual_type_by_object_members"); 
        let memo_key = (node.id(), contextual_type.id);
        if let Some(cached) = self.discriminated_contextual_types.get(&memo_key) {
            return Arc::clone(cached);
        }
        let NodeData::ObjectLiteralExpression(ol) = &node.data else {
            return Arc::clone(contextual_type);
        };
        let mut declared_names: HashSet<String> = HashSet::new();
        let mut items: Vec<(DiscriminantItem, String)> = Vec::new();
        for p in ol.properties.iter() {
            let name = match self.node_property_name(&p) {
                Some(n) => n,
                None => continue,
            };
            declared_names.insert(name.clone());
            match &p.data {
                NodeData::PropertyAssignment(pa) => {
                    if self.is_possibly_discriminant_value(&pa.initializer)
                        && self.is_discriminant_property(contextual_type, &name)
                    {
                        items.push((DiscriminantItem::PropertyAssignment(Arc::clone(&p)), name));
                    }
                }
                NodeData::ShorthandPropertyAssignment(_) => {
                    if self.is_discriminant_property(contextual_type, &name) {
                        items.push((DiscriminantItem::ShorthandProperty(Arc::clone(&p)), name));
                    }
                }
                _ => {}
            }
        }
        for prop in self.get_properties_of_type(contextual_type) {
            let Some(synthetic) = self.get_union_or_intersection_property(contextual_type, &prop.name)
            else {
                continue;
            };
            if !synthetic.flags.contains(SymbolFlags::Optional)
                || declared_names.contains(&prop.name)
                || !self.is_discriminant_property(contextual_type, &prop.name)
            {
                continue;
            }
            items.push((DiscriminantItem::MissingMember, prop.name.clone()));
        }
        let discriminated = self.discriminate_type_by_discriminable_items(contextual_type, items);
        // Go checker.go:31132：nil 判别结果写入 map 是 no-op（命中判定以
        // 非 nil 为准），失败不驻留，后续查询可重算；Rust 原 None 驻留会把
        // 早期查询（合成判别属性未就绪等）的失败结果固化，污染同
        // (node, type) 键的后续判别（discriminantPropertyInference
        // case3 错 case4 对的时序悖论根因）。重入递归断路由
        // discriminant_item_matches 的上下文无关取型承担（Go 同构）
        if !Arc::ptr_eq(&discriminated, contextual_type) {
            self.discriminated_contextual_types
                .insert(memo_key, Arc::clone(&discriminated));
        }
        discriminated
    }

    /// Go discriminateTypeByDiscriminableItems（relater.go:1205）
    fn discriminate_type_by_discriminable_items(
        &mut self,
        target: &Arc<Type>,
        items: Vec<(DiscriminantItem, String)>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("discriminate_type_by_discriminable_items"); 
        let Some(types) = target.types() else {
            return Arc::clone(target);
        };
        let types = types.to_vec();
        let primitive = type_flags_primitive();
        let mut include: Vec<i8> = types
            .iter()
            .map(|t| {
                (!t.flags.intersects(primitive)
                    && !self
                        .discriminant_reduced_type(t)
                        .flags
                        .contains(TypeFlags::Never)) as i8
            })
            .collect();
        for (item, name) in &items {
            let mut matched = false;
            for i in 0..types.len() {
                if include[i] != FALSE {
                    if let Some(target_type) =
                        self.type_of_property_or_index_signature(&types[i], name)
                    {
                        if self.discriminant_item_matches(item, &target_type) {
                            matched = true;
                        } else {
                            include[i] = MAYBE;
                        }
                    }
                }
            }
            for state in include.iter_mut() {
                if *state == MAYBE {
                    *state = if matched { FALSE } else { TRUE };
                }
            }
        }
        if include.contains(&FALSE) {
            let filtered: Vec<Arc<Type>> = types
                .iter()
                .zip(&include)
                .filter(|(_, s)| **s == TRUE)
                .map(|(t, _)| Arc::clone(t))
                .collect();
            let filtered = self.get_union_type(filtered);
            if !filtered.flags.contains(TypeFlags::Never) {
                return filtered;
            }
        }
        Arc::clone(target)
    }

    /// Go getReducedType（checker.go:22161）：本处仅用于成分预过滤，
    /// never 交叉归约外联合内部归约（ContainsIntersections）未移植
    fn discriminant_reduced_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("discriminant_reduced_type"); 
        if t.flags.contains(TypeFlags::Intersection) && self.is_never_intersection(t) {
            return self.never_type();
        }
        Arc::clone(t)
    }

    /// Go getTypeOfPropertyOrIndexSignatureOfType
    fn type_of_property_or_index_signature(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("type_of_property_or_index_signature"); 
        if let Some(prop) = self.get_property_of_type(t, name) {
            return Some(self.get_type_of_symbol(&prop));
        }
        if !crate::checker::utilities_is_optional_symbol::is_late_bound_name(name) {
            let name_literal = self.get_string_literal_type(name);
            if let Some(info) = self.get_applicable_index_info(t, &name_literal) {
                return info.value_type.clone();
            }
        }
        None
    }

    /// Go ObjectLiteralDiscriminator.matches（checker.go:31081）：属性赋值
    /// 取初始化式**上下文无关**型（getContextFreeTypeOfExpression——判别式
    /// 限定于不依赖上下文型的表达式即为此递归断路），简写属性取名标识符
    /// 型，缺席成员以 undefined 参与判定
    fn discriminant_item_matches(&mut self, item: &DiscriminantItem, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("discriminant_item_matches"); 
        let prop_type = match item {
            DiscriminantItem::PropertyAssignment(p) => match &p.data {
                NodeData::PropertyAssignment(pa) => {
                    self.get_context_free_type_of_expression(&pa.initializer)
                }
                _ => return false,
            },
            DiscriminantItem::ShorthandProperty(p) => match &p.data {
                NodeData::ShorthandPropertyAssignment(sa) => {
                    self.get_context_free_type_of_expression(&sa.name)
                }
                _ => return false,
            },
            DiscriminantItem::MissingMember => self.undefined_type(),
        };
        distributed(&prop_type)
            .into_iter()
            .any(|s| self.is_type_assignable_to(&s, t))
    }
}
