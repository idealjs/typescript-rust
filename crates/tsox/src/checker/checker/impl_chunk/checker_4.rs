#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn boolean_type(&self) -> Arc<Type> {
        self.boolean_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Boolean,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "boolean".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn es_symbol_type(&self) -> Arc<Type> {
        self.es_symbol_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::ESSymbol,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "symbol".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn void_type(&self) -> Arc<Type> {
        self.void_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Void,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "void".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn never_type(&self) -> Arc<Type> {
        self.never_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Never,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "never".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn auto_type(&self) -> Arc<Type> {
        self.auto_type
            .get_or_init(|| {
                Arc::new(Type {
                    flags: TypeFlags::Any,
                    object_flags: ObjectFlags::NonInferrableType,
                    id: crate::checker::types::next_type_id(),
                    symbol: None,
                    alias: None,
                    data: TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "any".to_string(),
                    }),
                })
            })
            .clone()
    }

    pub fn auto_array_type(&mut self) -> Arc<Type> {
        if let Some(t) = self.auto_array_type.get() {
            return Arc::clone(t);
        }
        let auto = self.auto_type();
        let arr = self.create_array_type(auto);

        self.auto_array_type
            .set(arr.clone())
            .ok()
            .map(|()| arr.clone())
            .unwrap_or_else(|| self.auto_array_type.get().cloned().unwrap_or(arr))
    }

    pub fn get_evolving_array_type(&mut self, element_type: Arc<Type>) -> Arc<Type> {
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::EvolvingArray,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::EvolvingArray(EvolvingArrayTypeData {
                object: ObjectTypeData::default(),
                element_type: Some(element_type),
                final_array_type: OnceLock::new(),
            }),
        })
    }

    pub fn add_evolving_array_element_type(
        &mut self,
        evolving_type: &Arc<Type>,
        new_element_type: Arc<Type>,
    ) -> Arc<Type> {
        let current_element = match &evolving_type.data {
            TypeData::EvolvingArray(ea) => ea.element_type.clone(),
            _ => return Arc::clone(evolving_type),
        };
        match current_element {
            Some(current) => {
                if self.is_type_subset_of(&new_element_type, &current) {
                    return Arc::clone(evolving_type);
                }
                let union = self.get_union_type(vec![current, new_element_type]);
                self.get_evolving_array_type(union)
            }
            None => self.get_evolving_array_type(new_element_type),
        }
    }

    pub fn finalize_evolving_array_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if !t.object_flags.contains(ObjectFlags::EvolvingArray) {
            return Arc::clone(t);
        }
        match &t.data {
            TypeData::EvolvingArray(ea) => {
                if let Some(final_t) = ea.final_array_type.get() {
                    return Arc::clone(final_t);
                }
                let element = ea.element_type.clone().unwrap_or_else(|| self.never_type());
                let result = if element.flags.contains(TypeFlags::Never) {
                    self.auto_array_type()
                } else if element.flags.contains(TypeFlags::Union) {
                    self.create_array_type(element)
                } else {
                    self.create_array_type(element)
                };

                if let TypeData::EvolvingArray(ea) = &t.data {
                    let _ = ea.final_array_type.set(Arc::clone(&result));
                }
                result
            }
            _ => Arc::clone(t),
        }
    }

    pub fn is_type_subset_of(&mut self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        if Arc::ptr_eq(a, b) || self.types_are_equal(a, b) {
            return true;
        }
        if a.flags.contains(TypeFlags::Never) {
            return true;
        }
        if b.flags.contains(TypeFlags::Any) || b.flags.contains(TypeFlags::Unknown) {
            return true;
        }

        self.is_type_assignable_to(a, b)
    }

    pub fn any_function_type(&self) -> Arc<Type> {
        self.any_function_type
            .get_or_init(|| {
                Arc::new(Type {
                    flags: TypeFlags::Object,
                    object_flags: ObjectFlags::Anonymous,
                    id: crate::checker::types::next_type_id(),
                    symbol: None,
                    alias: None,
                    data: TypeData::Object(ObjectTypeData::default()),
                })
            })
            .clone()
    }

    pub fn non_primitive_type(&self) -> Arc<Type> {
        self.non_primitive_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::NonPrimitive,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "object".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn true_type(&self) -> Arc<Type> {
        self.true_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::BooleanLiteral,
                    TypeData::Literal(LiteralTypeData {
                        value: LiteralValue::Boolean(true),
                        fresh_type: OnceLock::new(),
                        regular_type: OnceLock::new(),
                    }),
                ))
            })
            .clone()
    }

    pub fn false_type(&self) -> Arc<Type> {
        self.false_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::BooleanLiteral,
                    TypeData::Literal(LiteralTypeData {
                        value: LiteralValue::Boolean(false),
                        fresh_type: OnceLock::new(),
                        regular_type: OnceLock::new(),
                    }),
                ))
            })
            .clone()
    }

    pub fn error_type(&self) -> Arc<Type> {
        self.error_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Any,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "error".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn get_string_literal_type(&mut self, value: &str) -> Arc<Type> {
        if let Some(t) = self.string_literal_types.get(value) {
            return Arc::clone(t);
        }
        let t = Arc::new(Type::new(
            TypeFlags::StringLiteral,
            TypeData::Literal(LiteralTypeData {
                value: LiteralValue::String(value.to_string()),
                fresh_type: OnceLock::new(),
                regular_type: OnceLock::new(),
            }),
        ));
        self.string_literal_types
            .insert(value.to_string(), Arc::clone(&t));
        t
    }

    pub fn get_number_literal_type(&mut self, value: jsnum::Number) -> Arc<Type> {
        if let Some(t) = self.number_literal_types.get(&value) {
            return Arc::clone(t);
        }
        let t = Arc::new(Type::new(
            TypeFlags::NumberLiteral,
            TypeData::Literal(LiteralTypeData {
                value: LiteralValue::Number(value),
                fresh_type: OnceLock::new(),
                regular_type: OnceLock::new(),
            }),
        ));
        self.number_literal_types.insert(value, Arc::clone(&t));
        t
    }
}
