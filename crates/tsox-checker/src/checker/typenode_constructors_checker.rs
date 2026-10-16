#![allow(unused_imports)]

use crate::checker::typenode_constructors::*;
use crate::checker::types::{IntersectionFlags, UnionReduction};

impl Checker {
    pub(crate) fn get_optional_type(&mut self, t: Arc<Type>, is_property: bool) -> Arc<Type> { ::tsox_core::fntrace::enter("get_optional_type");
        let missing_or_undefined = if is_property {
            self.undefined_or_missing_type()
        } else {
            self.undefined_type()
        };
        if Arc::ptr_eq(&t, &missing_or_undefined)
            || t.flags.intersects(TypeFlags::Union)
                && t.types()
                    .is_some_and(|types| Arc::ptr_eq(&types[0], &missing_or_undefined))
        {
            return t;
        }
        self.get_union_type(vec![t, missing_or_undefined])
    }

    pub(crate) fn get_union_type(&mut self, types: Vec<Arc<Type>>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_union_type");
        self.get_union_type_ex(types, UnionReduction::Literal, None, None)
    }

    pub(crate) fn get_intersection_type(&mut self, types: Vec<Arc<Type>>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_intersection_type");
        // Go getIntersectionType（checker.go:27603-27606）单点委托
        // getIntersectionTypeEx：位聚合（addTypeToIntersection）、空对象超型
        // 剔除、约束消解、交叉积与别名承载全在 faithful 单实现
        // （mig/m1e_3.rs:250），简化双实现收敛为一
        self.get_intersection_type_ex(&types, IntersectionFlags::None, None)
    }

    pub(crate) fn create_array_type(&mut self, element_type: Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("create_array_type"); 
        self.create_array_type_ex(element_type, false)
    }

    /// readonly T[] 的数组形态：带 IsReadonlyArray 标志的数组实例（tsc 映射为
    /// ReadonlyArray<T> 引用；这里以标志承载 readonly 语义，成员仍共用 Array 声明，
    /// 关系判定按元素协变 + readonly→可变拒绝处理）
    pub(crate) fn create_array_type_ex(
        &mut self,
        element_type: Arc<Type>,
        readonly: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("create_array_type_ex"); 
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
        object_flags |= self.get_propagating_flags_of_types(
            std::slice::from_ref(&element_type),
            TypeFlags::empty(),
        );
        let array_type = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags,
            id: crate::checker::types::next_type_id(),
            symbol: Some(array_symbol),
            alias: None,
            data: TypeData::Object(ObjectTypeData { node: None,
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

    pub(crate) fn array_type_parameter_symbols(&mut self) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("array_type_parameter_symbols"); 
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
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("instantiate_array_member_type"); 
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
        // Go createTupleTargetType（checker.go:26264-26274）：length 无条件进驻
        // tuple target 的 declaredMembers，自有成员先于 Array 基类成员
        //（定长为字面量并集、变长为 number），不得回落 Array.length
        if is_tuple && member.name == "length" {
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

    pub(crate) fn tuple_base_element_type(&mut self, tuple: &TupleTypeData) -> Arc<Type> { ::tsox_core::fntrace::enter("tuple_base_element_type"); 
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

    pub(crate) fn declared_array_member_symbol(&mut self, name: &str) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("declared_array_member_symbol"); 
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

    pub(crate) fn declared_array_member_symbols(&mut self) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("declared_array_member_symbols"); 
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
    ) -> Option<Vec<Arc<Symbol>>> { ::tsox_core::fntrace::enter("global_interface_properties"); 
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
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("global_interface_member_symbol"); 
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

pub(crate) fn uniform_enum_symbol(types: &[Arc<Type>]) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("uniform_enum_symbol"); 
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
