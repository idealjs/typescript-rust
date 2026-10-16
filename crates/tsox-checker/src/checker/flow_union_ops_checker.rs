#![allow(unused_imports)]

use crate::checker::flow_union_ops::*;

impl Checker {
    pub(crate) fn constituent_is_definitely_falsy(&self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("constituent_is_definitely_falsy"); 
        if t.flags.intersects(
            TypeFlags::Undefined
                | TypeFlags::Null
                | TypeFlags::Void
                | TypeFlags::Any
                | TypeFlags::Unknown
                | TypeFlags::Never,
        ) {
            return true;
        }
        if t.flags.contains(TypeFlags::BooleanLiteral) {
            return matches!(
                t.literal_value(),
                Some(crate::checker::types::LiteralValue::Boolean(false))
            );
        }
        if t.flags.contains(TypeFlags::StringLiteral) {
            return t
                .intrinsic_name()
                .is_some_and(|n| n == "\"\"" || n.is_empty());
        }
        if t.flags.contains(TypeFlags::NumberLiteral) {
            return t.intrinsic_name().is_some_and(|n| n == "0");
        }
        false
    }

    pub(crate) fn flow_constituents_public(&self, t: &Arc<Type>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("flow_constituents_public"); 
        self.constituent_types(t)
    }

    pub(crate) fn flow_constituent_definitely_falsy(&self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("flow_constituent_definitely_falsy"); 
        self.constituent_is_definitely_falsy(t)
    }

    fn union_with_boolean_merge(&self, members: Vec<Arc<Type>>) -> Arc<Type> { ::tsox_core::fntrace::enter("union_with_boolean_merge");
        // Go filterType/removeType 重建并集统一走 getUnionTypeFromSortedList
        // （checker.go:28162/28180），双成员均为 boolean 字面量时置
        // TypeFlagsBoolean（checker.go:27291-27292），按 boolean 关键字呈现
        let is_boolean_union = members.len() == 2
            && members[0].flags.contains(TypeFlags::BooleanLiteral)
            && members[1].flags.contains(TypeFlags::BooleanLiteral);
        let mut union = Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: members,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        );
        if is_boolean_union {
            union.flags.insert(TypeFlags::Boolean);
        }
        Arc::new(union)
    }

    pub(crate) fn extract_definitely_falsy_constituents(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("extract_definitely_falsy_constituents"); 
        let falsy: Vec<Arc<Type>> = self
            .constituent_types(t)
            .into_iter()
            .filter(|c| self.constituent_is_definitely_falsy(c))
            .collect();
        self.rebuild_union_or_never(t, falsy)
    }

    pub(crate) fn remove_definitely_falsy_constituents(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("remove_definitely_falsy_constituents");
        let kept: Vec<Arc<Type>> = self
            .constituent_types(t)
            .into_iter()
            .filter(|c| !self.constituent_is_definitely_falsy(c))
            .collect();
        if kept.is_empty() {
            // Go removeDefinitelyFalsyTypes = filterType(t, hasTruthy)
            // （checker.go:30815-30818）：成员全 falsy 时空并集归 neverType
            // （checker.go:27278-27280），非联合成员不满足谓词同落 never
            // （checker.go:28165-28168）。`false || x` 类 left 的结果为
            // union(never, x) = x 而非保留原 falsy 型
            return self.never_type();
        }
        self.rebuild_union_or_never(t, kept)
    }

    pub(crate) fn flow_union_of(&self, types: &[Arc<Type>]) -> Arc<Type> { ::tsox_core::fntrace::enter("flow_union_of"); 
        let mut all: Vec<Arc<Type>> = Vec::new();
        for t in types {
            for c in self.constituent_types(t) {
                // Go 成员扁平化展开 booleanType（checker.go:1013
                // booleanType = getUnionType([regularFalseType, regularTrueType])）：
                // 内建 boolean 单例按 false/true 字面量成员入并
                if c.flags.contains(TypeFlags::Boolean) && !c.is_union() {
                    for b in [self.false_type(), self.true_type()] {
                        if !all.iter().any(|s| Arc::ptr_eq(s, &b)) {
                            all.push(b);
                        }
                    }
                } else if !all.iter().any(|s| Arc::ptr_eq(s, &c)) {
                    all.push(c);
                }
            }
        }
        if all.is_empty() {
            return self.never_type();
        }
        if all.len() == 1 {
            return all.into_iter().next().expect("exactly one");
        }
        // Go addTypesToUnion（checker.go:27335-27341）len>=2 排序：流收窄并集
        // （narrow_by_expression 的 ||/&& 分支并集、truthiness/逻辑赋值归约）
        // 成员序按 CompareTypes（flags→名称→符号→结构→id），不保留构造序
        all.sort_by(|a, b| self.compare_union_members_ordered(a, b));
        // Go getUnionTypeFromSortedList（checker.go:27290-27292）：恰为
        // false/true 双字面量成员的联合置 TypeFlagsBoolean，按 boolean
        // 关键字呈现而非 false | true
        let is_boolean_union = all.len() == 2
            && all[0].flags.contains(TypeFlags::BooleanLiteral)
            && all[1].flags.contains(TypeFlags::BooleanLiteral);
        let mut union = Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: all,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        );
        if is_boolean_union {
            union.flags.insert(TypeFlags::Boolean);
        }
        Arc::new(union)
    }

    pub(crate) fn remove_type_from_union(
        &self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("remove_type_from_union"); 
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| !self.types_overlap(t, value_type))
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }

        self.union_with_boolean_merge(remaining)
    }

    pub fn remove_flags_from_union(&self, type_: &Arc<Type>, flags: TypeFlags) -> Arc<Type> { ::tsox_core::fntrace::enter("remove_flags_from_union"); 
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| !t.flags.intersects(flags))
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        self.union_with_boolean_merge(remaining)
    }

    pub(crate) fn filter_type_by_flags(&self, type_: &Arc<Type>, flags: TypeFlags) -> Arc<Type> { ::tsox_core::fntrace::enter("filter_type_by_flags"); 
        let constituents = self.constituent_types(type_);
        let matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| t.flags.intersects(flags))
            .collect();
        if matching.is_empty() {
            return self.never_type();
        }
        if matching.len() == 1 {
            return matching.into_iter().next().expect("exactly one");
        }
        self.union_with_boolean_merge(matching)
    }

    pub(crate) fn filter_type_by_object(&self, type_: &Arc<Type>, is_loose: bool) -> Arc<Type> { ::tsox_core::fntrace::enter("filter_type_by_object"); 
        let constituents = self.constituent_types(type_);
        let mut matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                t.flags.contains(TypeFlags::Object)
                    || t.flags.contains(TypeFlags::Null)
                    || (is_loose && t.flags.contains(TypeFlags::Undefined))
            })
            .collect();
        if matching.is_empty() {
            return self.never_type();
        }
        if matching.len() == 1 {
            return matching.into_iter().next().expect("exactly one");
        }
        self.union_with_boolean_merge(matching.drain(..).collect())
    }

    pub(crate) fn filter_type_by_callable(
        &self,
        type_: &Arc<Type>,
        keep_callable: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("filter_type_by_callable"); 
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let is_callable = !self
                    .get_signatures_of_type(t, SignatureKind::Call)
                    .is_empty();
                if keep_callable {
                    is_callable
                } else {
                    !is_callable
                }
            })
            .collect();
        if filtered.is_empty() {
            return self.never_type();
        }
        if filtered.len() == 1 {
            return filtered.into_iter().next().expect("exactly one");
        }
        self.union_with_boolean_merge(filtered)
    }

    pub(crate) fn remove_object_from_union(&self, type_: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("remove_object_from_union"); 
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| !t.flags.contains(TypeFlags::Object) && !t.flags.contains(TypeFlags::Null))
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        self.union_with_boolean_merge(remaining)
    }
}
