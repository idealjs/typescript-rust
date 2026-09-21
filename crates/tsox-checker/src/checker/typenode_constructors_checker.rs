#![allow(unused_imports)]

use crate::checker::typenode_constructors::*;

impl Checker {
    pub(crate) fn get_optional_type(&mut self, t: Arc<Type>) -> Arc<Type> {
        if self.strict_null_checks && !self.exact_optional_property_types {
            self.get_union_type(vec![t, self.undefined_type()])
        } else {
            t
        }
    }

    pub(crate) fn get_union_type(&mut self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.never_type();
        }
        let mut has_unknown = false;
        for t in &types {
            if t.flags.contains(TypeFlags::Any) {
                return self.any_type();
            }
            if t.flags.contains(TypeFlags::Unknown) {
                has_unknown = true;
            }
        }
        if has_unknown {
            return self.unknown_type();
        }

        let types: Vec<Arc<Type>> = types
            .into_iter()
            .filter(|t| !t.flags.contains(TypeFlags::Never))
            .collect();
        if types.is_empty() {
            return self.never_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }

        let mut flattened: Vec<Arc<Type>> = Vec::with_capacity(types.len());
        for t in types {
            if let TypeData::Union(u) = &t.data {
                for inner in &u.union_or_intersection.types {
                    if !flattened.iter().any(|s| Arc::ptr_eq(s, inner)) {
                        flattened.push(Arc::clone(inner));
                    }
                }
            } else if !flattened.iter().any(|s| Arc::ptr_eq(s, &t)) {
                flattened.push(t);
            }
        }
        if flattened.is_empty() {
            return self.never_type();
        }
        if flattened.len() == 1 {
            return flattened.into_iter().next().expect("exactly one");
        }

        {
            let rank = |t: &Arc<Type>| -> u32 {
                if t.flags.intersects(TypeFlags::EnumLiteral | TypeFlags::Enum) {
                    return TypeFlags::Enum.bits();
                }
                let b = t.flags.bits();
                b & b.wrapping_neg()
            };
            flattened.sort_by_key(rank);
        }
        // Go getUnionType 默认 Literal 归约（removeSubtypes）：字面量成员被
        // 同族原始类型成员吸收（"john" | string → string）
        let has_primitive = |ts: &[Arc<Type>], flag: TypeFlags| {
            ts.iter()
                .any(|t| t.flags.contains(flag) && !crate::checker::types::TYPE_FLAGS_LITERAL.contains(t.flags))
        };
        let literal_base = |t: &Arc<Type>| -> Option<TypeFlags> {
            for (lit, prim) in [
                (TypeFlags::StringLiteral, TypeFlags::String),
                (TypeFlags::NumberLiteral, TypeFlags::Number),
                (TypeFlags::BigIntLiteral, TypeFlags::BigInt),
                (TypeFlags::BooleanLiteral, TypeFlags::Boolean),
            ] {
                if t.flags.contains(lit) {
                    return Some(prim);
                }
            }
            None
        };
        let absorb: Vec<TypeFlags> = {
            let mut v = Vec::new();
            for t in &flattened {
                if let Some(prim) = literal_base(t)
                    && has_primitive(&flattened, prim)
                    && !v.contains(&prim)
                {
                    v.push(prim);
                }
            }
            v
        };
        if !absorb.is_empty() {
            flattened.retain(|t| match literal_base(t) {
                Some(prim) => !absorb.contains(&prim),
                None => true,
            });
        }
        if flattened.len() == 1 {
            return flattened.into_iter().next().expect("exactly one");
        }
        // 全体成员都是同一枚举的字面量时，联合型挂枚举符号（消息渲染按符号名）
        let enum_symbol = uniform_enum_symbol(&flattened);
        let mut union = Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: flattened,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        );
        union.symbol = enum_symbol;
        Arc::new(union)
    }

    pub(crate) fn get_intersection_type(&mut self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.unknown_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }
        // Go createIntersectionType：相同/结构等价的成分去重（同一类型自交化简）
        let mut deduped: Vec<Arc<Type>> = Vec::with_capacity(types.len());
        for t in types {
            if !deduped.iter().any(|d| shallow_type_eq(d, &t)) {
                deduped.push(t);
            }
        }
        if deduped.len() == 1 {
            return deduped.into_iter().next().expect("exactly one");
        }
        let mut includes = TypeFlags::empty();
        for t in &deduped {
            includes.insert(t.flags);
        }
        if includes.contains(TypeFlags::Never) {
            return self.never_type();
        }
        let disjoint_domains = TYPE_FLAGS_DISJOINT_DOMAINS;
        let without = |f: TypeFlags| TypeFlags::from_bits_truncate(disjoint_domains.bits() & !f.bits());
        if self.strict_null_checks
            && includes.intersects(TYPE_FLAGS_NULLABLE)
            && includes.intersects(TypeFlags::Object | TypeFlags::NonPrimitive)
            || includes.contains(TypeFlags::NonPrimitive)
                && includes.intersects(without(TypeFlags::NonPrimitive))
            || includes.intersects(TYPE_FLAGS_STRING_LIKE)
                && includes.intersects(without(TYPE_FLAGS_STRING_LIKE))
            || includes.intersects(TYPE_FLAGS_NUMBER_LIKE)
                && includes.intersects(without(TYPE_FLAGS_NUMBER_LIKE))
            || includes.intersects(TYPE_FLAGS_BIG_INT_LIKE)
                && includes.intersects(without(TYPE_FLAGS_BIG_INT_LIKE))
            || includes.intersects(TYPE_FLAGS_ES_SYMBOL_LIKE)
                && includes.intersects(without(TYPE_FLAGS_ES_SYMBOL_LIKE))
            || includes.intersects(TYPE_FLAGS_VOID_LIKE)
                && includes.intersects(without(TYPE_FLAGS_VOID_LIKE))
        {
            return self.never_type();
        }
        if includes.contains(TypeFlags::Any) {
            return self.any_type();
        }
        if !self.strict_null_checks && includes.intersects(TYPE_FLAGS_NULLABLE) {
            if includes.contains(TypeFlags::Undefined) {
                return self.undefined_type();
            }
            return self.null_type();
        }
        // Go removeRedundantSupertypes：字面量/模板/映射型存在时删除同域
        // 原始超类型（"q" & string → "q"）
        let reducible = includes.contains(TypeFlags::String)
            && includes.intersects(
                TypeFlags::StringLiteral | TypeFlags::TemplateLiteral | TypeFlags::StringMapping,
            )
            || includes.contains(TypeFlags::Number) && includes.intersects(TypeFlags::NumberLiteral)
            || includes.contains(TypeFlags::BigInt) && includes.intersects(TypeFlags::BigIntLiteral)
            || includes.contains(TypeFlags::ESSymbol)
                && includes.intersects(TypeFlags::UniqueESSymbol)
            || includes.contains(TypeFlags::Void) && includes.intersects(TypeFlags::Undefined);
        let mut deduped = deduped;
        if reducible {
            deduped.retain(|t| {
                let supertype_of_literal = (t.flags.contains(TypeFlags::String)
                    && includes.intersects(
                        TypeFlags::StringLiteral
                            | TypeFlags::TemplateLiteral
                            | TypeFlags::StringMapping,
                    ))
                    || (t.flags.contains(TypeFlags::Number)
                        && includes.intersects(TypeFlags::NumberLiteral))
                    || (t.flags.contains(TypeFlags::BigInt)
                        && includes.intersects(TypeFlags::BigIntLiteral))
                    || (t.flags.contains(TypeFlags::ESSymbol)
                        && includes.intersects(TypeFlags::UniqueESSymbol))
                    || (t.flags.contains(TypeFlags::Void)
                        && includes.intersects(TypeFlags::Undefined));
                !supertype_of_literal
            });
            if deduped.len() == 1 {
                return deduped.into_iter().next().expect("exactly one");
            }
        }
        Arc::new(Type::new(
            TypeFlags::Intersection,
            TypeData::Intersection(IntersectionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: deduped,
                },
                resolved_apparent_type: std::sync::OnceLock::new(),
                unique_literal_filled_instantiation: std::sync::OnceLock::new(),
                resolved_properties: std::sync::OnceLock::new(),
            }),
        ))
    }

    pub(crate) fn create_array_type(&mut self, element_type: Arc<Type>) -> Arc<Type> {
        self.create_array_type_ex(element_type, false)
    }

    /// readonly T[] 的数组形态：带 IsReadonlyArray 标志的数组实例（tsc 映射为
    /// ReadonlyArray<T> 引用；这里以标志承载 readonly 语义，成员仍共用 Array 声明，
    /// 关系判定按元素协变 + readonly→可变拒绝处理）
    pub(crate) fn create_array_type_ex(
        &mut self,
        element_type: Arc<Type>,
        readonly: bool,
    ) -> Arc<Type> {
        let Some(array_symbol) = self.globals.get("Array").cloned() else {
            return self.get_any_type();
        };
        // 按元素实例驻留（tsc getTypeFromArrayType）：同一元素类型只建一个
        // 数组实例，关系判定的进行中配对检测才能命中循环引用
        // （never[] → ReadonlyArray<number> 走 every/flatMap 成员结构比较时
        // 会递归回到同一对类型）
        let intern_key = (element_type.id, readonly);
        if let Some(cached) = self.array_type_intern_cache.get(&intern_key) {
            return Arc::clone(cached);
        }
        let target = self.get_declared_type_of_symbol(&array_symbol);
        let mut object_flags = ObjectFlags::Reference;
        if readonly {
            object_flags |= ObjectFlags::IsReadonlyArray;
        }
        let array_type = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags,
            id: crate::checker::types::next_type_id(),
            symbol: Some(array_symbol),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData::default(),
                target: Some(target),
                mapper: None,
                type_arguments: vec![element_type],
            }),
        });
        self.array_type_intern_cache
            .insert(intern_key, Arc::clone(&array_type));
        array_type
    }

    pub(crate) fn array_type_parameter_symbols(&mut self) -> Vec<Arc<Symbol>> {
        if let Some(cached) = &self.array_type_parameter_symbols {
            return cached.clone();
        }
        // 全部 interface 声明的类型参数符号（es5 主声明 + es2015+ 各增强
        // 文件的同名 T 都是独立符号，代入须一网打尽）
        let collected = self
            .globals
            .get("Array")
            .map(|sym| {
                let sym_map = self.program.symbol_map();
                sym.declarations
                    .iter()
                    .filter_map(|decl| match &decl.data {
                        NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
                        _ => None,
                    })
                    .flat_map(|tps| tps.iter())
                    .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        self.array_type_parameter_symbols = Some(collected.clone());
        collected
    }

    pub(crate) fn instantiate_array_member_type(
        &mut self,
        obj_type: &Arc<Type>,
        member: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {
        let is_evolving = obj_type.object_flags.contains(ObjectFlags::EvolvingArray);
        let is_tuple = matches!(&obj_type.data, TypeData::Tuple(_));
        if !self.is_array_type(obj_type) && !is_evolving && !is_tuple {
            return None;
        }
        if let Some(structured) = obj_type.as_structured()
            && structured.members.get(&member.name).is_some()
        {
            return None;
        }
        let element = match &obj_type.data {
            TypeData::Object(o) => match o.type_arguments.first() {
                Some(e) => Arc::clone(e),
                None => return None,
            },
            TypeData::EvolvingArray(e) => {
                e.element_type.clone().unwrap_or_else(|| self.never_type())
            }
            // Go getTupleBaseType：元素并集作 Array 的类型实参（变长元素按
            // number 索引访问拍平）
            TypeData::Tuple(t) => self.tuple_base_element_type(t),
            _ => return None,
        };

        let declared = match self.globals.get("Array").and_then(|sym| {
            self.type_alias_links
                .get(sym)
                .and_then(|l| l.declared_type.clone())
        }) {
            Some(d) => Some(d),

            None => self
                .globals
                .get("Array")
                .cloned()
                .map(|sym| self.resolve_interface_type(&sym, None)),
        };
        let raw = declared
            .as_ref()
            .and_then(|d| d.as_structured())
            .and_then(|s| s.members.get(&member.name).cloned())
            .map(|synthetic| self.get_type_of_symbol(&synthetic))?;

        let key = (
            Arc::as_ptr(&element) as *const crate::checker::types::Type as usize,
            Arc::as_ptr(member) as *const tsox_frontend::ast::Symbol as usize,
        );
        if let Some(cached) = self.array_member_type_cache.get(&key) {
            return Some(Arc::clone(cached));
        }

        let mut free_tps: Vec<Arc<Type>> = Vec::new();
        for sig in self.get_signatures_of_type(&raw, SignatureKind::Call) {
            for param in &sig.parameters {
                let pt = self.get_type_of_symbol(param);
                self.collect_free_type_parameters_deep(&pt, &mut free_tps);
            }
            if let Some(rt) = self.get_return_type_of_signature(&sig) {
                self.collect_free_type_parameters_deep(&rt, &mut free_tps);
            }
        }

        let array_tps = self.array_type_parameter_symbols();
        let subst_tps: Vec<Arc<Type>> = free_tps
            .iter()
            .filter(|tp| {
                tp.symbol
                    .as_ref()
                    .is_some_and(|s| array_tps.iter().any(|a| Arc::ptr_eq(a, s)))
            })
            .cloned()
            .collect();
        if subst_tps.is_empty() {
            return Some(raw);
        }
        let substitutions: Vec<Arc<Type>> = std::iter::repeat(Arc::clone(&element))
            .take(subst_tps.len())
            .collect();
        let substituted = self.substitute_infer_type_parameters(&raw, &subst_tps, &substitutions);
        self.array_member_type_cache
            .insert(key, Arc::clone(&substituted));
        Some(substituted)
    }

    pub(crate) fn tuple_base_element_type(&mut self, tuple: &TupleTypeData) -> Arc<Type> {
        let number = self.number_type();
        let parts: Vec<Arc<Type>> = tuple
            .element_infos
            .iter()
            .filter_map(|info| {
                let t = info.type_.clone()?;
                if info.flags.contains(ElementFlags::Variadic) {
                    Some(self.get_indexed_access_type(&t, &number))
                } else {
                    Some(t)
                }
            })
            .collect();
        self.get_union_type(parts)
    }

    pub(crate) fn declared_array_member_symbol(&mut self, name: &str) -> Option<Arc<Symbol>> {
        let array_sym = self.globals.get("Array").cloned();
        let declared = array_sym
            .as_ref()
            .and_then(|sym| {
                self.type_alias_links
                    .get(sym)
                    .and_then(|l| l.declared_type.clone())
            })
            .or_else(|| {
                array_sym
                    .as_ref()
                    .map(|sym| self.resolve_interface_type(&sym, None))
            })?;
        declared
            .as_structured()
            .and_then(|s| s.members.get(name).cloned())
    }

    pub(crate) fn declared_array_member_symbols(&mut self) -> Vec<Arc<Symbol>> {
        let array_sym = self.globals.get("Array").cloned();
        let declared = array_sym
            .as_ref()
            .and_then(|sym| {
                self.type_alias_links
                    .get(sym)
                    .and_then(|l| l.declared_type.clone())
            })
            .or_else(|| {
                array_sym
                    .as_ref()
                    .map(|sym| self.resolve_interface_type(&sym, None))
            });
        declared
            .and_then(|t| t.as_structured().map(|s| s.properties.clone()))
            .unwrap_or_default()
    }

    /// 全局接口（String/Number/Boolean/Symbol）的实例成员（补全 apparent 用）
    pub fn global_interface_properties(
        &mut self,
        interface_name: &str,
    ) -> Option<Vec<Arc<Symbol>>> {
        let sym = self.globals.get(interface_name).cloned()?;
        let declared = self
            .type_alias_links
            .get(&sym)
            .and_then(|l| l.declared_type.clone())
            .or_else(|| Some(self.resolve_interface_type(&sym, None)))?;
        declared
            .as_structured()
            .map(|s| s.properties.clone())
    }

    pub(crate) fn global_interface_member_symbol(
        &mut self,
        interface_name: &str,
        member: &str,
    ) -> Option<Arc<Symbol>> {
        let sym = self.globals.get(interface_name).cloned()?;
        let declared = self
            .type_alias_links
            .get(&sym)
            .and_then(|l| l.declared_type.clone())
            .or_else(|| Some(self.resolve_interface_type(&sym, None)))?;
        declared
            .as_structured()
            .and_then(|s| s.members.get(member).cloned())
    }
}

pub(crate) fn uniform_enum_symbol(types: &[Arc<Type>]) -> Option<Arc<Symbol>> {
    let mut parent: Option<Arc<Symbol>> = None;
    let uniform = !types.is_empty() && types.iter().all(|t| {
        let Some(sym) = t
            .flags
            .intersects(TypeFlags::EnumLiteral)
            .then(|| t.symbol.clone())
            .flatten()
        else {
            return false;
        };
        let Some(p) = sym.parent() else {
            return false;
        };
        match &parent {
            None => {
                parent = Some(Arc::clone(&p));
                true
            }
            Some(prev) => Arc::ptr_eq(&p, prev),
        }
    });
    uniform.then_some(parent).flatten()
}

fn shallow_type_eq(a: &Type, b: &Type) -> bool {
    if a.id == b.id {
        return true;
    }
    if a.flags != b.flags
        || a.symbol.as_ref().map(|s| Arc::as_ptr(s))
            != b.symbol.as_ref().map(|s| Arc::as_ptr(s))
    {
        return false;
    }
    match (&a.data, &b.data) {
        (crate::checker::types::TypeData::Object(oa), crate::checker::types::TypeData::Object(ob)) => {
            oa.type_arguments.len() == ob.type_arguments.len()
                && oa
                    .type_arguments
                    .iter()
                    .zip(ob.type_arguments.iter())
                    .all(|(x, y)| x.id == y.id)
        }
        _ => false,
    }
}
