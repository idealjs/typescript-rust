use std::sync::Arc;

use crate::checker::checker::Checker;
use crate::checker::relater::RelationKind;
use crate::checker::types::*;

pub(crate) const TYPEOF_EQ_STRING: u32 = 1 << 0;
pub(crate) const TYPEOF_EQ_NUMBER: u32 = 1 << 1;
pub(crate) const TYPEOF_EQ_BIGINT: u32 = 1 << 2;
pub(crate) const TYPEOF_EQ_BOOLEAN: u32 = 1 << 3;
pub(crate) const TYPEOF_EQ_SYMBOL: u32 = 1 << 4;
pub(crate) const TYPEOF_EQ_OBJECT: u32 = 1 << 5;
pub(crate) const TYPEOF_EQ_FUNCTION: u32 = 1 << 6;
pub(crate) const TYPEOF_EQ_HOST_OBJECT: u32 = 1 << 7;
pub(crate) const TYPEOF_EQ_UNDEFINED: u32 = 1 << 8;
pub(crate) const TYPEOF_EQ_NULL: u32 = 1 << 9;

pub(crate) const TYPEOF_NE_STRING: u32 = 1 << 10;
pub(crate) const TYPEOF_NE_NUMBER: u32 = 1 << 11;
pub(crate) const TYPEOF_NE_BIGINT: u32 = 1 << 12;
pub(crate) const TYPEOF_NE_BOOLEAN: u32 = 1 << 13;
pub(crate) const TYPEOF_NE_SYMBOL: u32 = 1 << 14;
pub(crate) const TYPEOF_NE_OBJECT: u32 = 1 << 15;
pub(crate) const TYPEOF_NE_FUNCTION: u32 = 1 << 16;
pub(crate) const TYPEOF_NE_HOST_OBJECT: u32 = 1 << 17;
pub(crate) const TYPEOF_NE_UNDEFINED: u32 = 1 << 18;

const TYPEOF_FACTS_ALL: u32 = (1 << 19) - 1;
const TYPEOF_FACTS_UNKNOWN: u32 = TYPEOF_FACTS_ALL;
const TYPEOF_OR_FACTS_MASK: u32 = TYPEOF_EQ_FUNCTION | TYPEOF_NE_OBJECT;
const TYPEOF_AND_FACTS_MASK: u32 = TYPEOF_FACTS_ALL & !TYPEOF_OR_FACTS_MASK;
const TYPEOF_EQ_NULLABLE: u32 = TYPEOF_EQ_UNDEFINED | TYPEOF_EQ_NULL;
const TYPEOF_NE_ALL_EXCEPT_UNDEFINED: u32 = TYPEOF_NE_STRING
    | TYPEOF_NE_NUMBER
    | TYPEOF_NE_BIGINT
    | TYPEOF_NE_BOOLEAN
    | TYPEOF_NE_SYMBOL
    | TYPEOF_NE_OBJECT
    | TYPEOF_NE_FUNCTION
    | TYPEOF_NE_HOST_OBJECT;

impl Checker {
    pub(crate) fn narrow_type_by_type_name(
        &mut self,
        t: &Arc<Type>,
        type_name: &str,
    ) -> Arc<Type> {
        let implied = match type_name {
            "string" => self.string_type(),
            "number" => self.number_type(),
            "bigint" => self.bigint_type(),
            "boolean" => self.boolean_type(),
            "symbol" => self.es_symbol_type(),
            "undefined" => self.undefined_type(),
            _ => self.non_primitive_type(),
        };
        match type_name {
            "object" => {
                if t.flags.contains(TypeFlags::Any) {
                    return Arc::clone(t);
                }
                let non_null = self.narrow_type_by_typeof_facts(t, &implied, TYPEOF_EQ_OBJECT);
                let null = self.narrow_type_by_typeof_facts(t, &self.null_type(), TYPEOF_EQ_NULL);
                self.get_union_type(vec![non_null, null])
            }
            "function" => {
                if t.flags.contains(TypeFlags::Any) {
                    return Arc::clone(t);
                }
                match self.global_function_type_of("Function") {
                    Some(ft) => self.narrow_type_by_typeof_facts(t, &ft, TYPEOF_EQ_FUNCTION),
                    None => Arc::clone(t),
                }
            }
            "string" | "number" | "bigint" | "boolean" | "symbol" => {
                let fact = match type_name {
                    "string" => TYPEOF_EQ_STRING,
                    "number" => TYPEOF_EQ_NUMBER,
                    "bigint" => TYPEOF_EQ_BIGINT,
                    "boolean" => TYPEOF_EQ_BOOLEAN,
                    _ => TYPEOF_EQ_SYMBOL,
                };
                self.narrow_type_by_typeof_facts(t, &implied, fact)
            }
            "undefined" => self.narrow_type_by_typeof_facts(t, &implied, TYPEOF_EQ_UNDEFINED),
            _ => self.narrow_type_by_typeof_facts(t, &implied, TYPEOF_EQ_HOST_OBJECT),
        }
    }

    pub(crate) fn narrow_type_by_typeof_facts(
        &mut self,
        t: &Arc<Type>,
        implied: &Arc<Type>,
        fact: u32,
    ) -> Arc<Type> {
        if t.is_union() {
            let mapped: Vec<Arc<Type>> = self
                .constituent_types(t)
                .into_iter()
                .map(|c| self.narrow_constituent_by_typeof_facts(&c, implied, fact))
                .filter(|c| !c.flags.contains(TypeFlags::Never))
                .collect();
            return self.rebuild_union_or_never(t, mapped);
        }
        self.narrow_constituent_by_typeof_facts(t, implied, fact)
    }

    fn narrow_constituent_by_typeof_facts(
        &mut self,
        t: &Arc<Type>,
        implied: &Arc<Type>,
        fact: u32,
    ) -> Arc<Type> {
        if self.is_type_related_to(t, implied, RelationKind::StrictSubtype) {
            if self.get_typeof_facts(t) & fact != 0 {
                return Arc::clone(t);
            }
            return self.never_type();
        }
        if self.is_type_subtype_of(implied, t) {
            return Arc::clone(implied);
        }
        if self.get_typeof_facts(t) & fact != 0 {
            return self.get_intersection_type(vec![Arc::clone(t), Arc::clone(implied)]);
        }
        self.never_type()
    }

    pub(crate) fn get_typeof_facts(&mut self, t: &Arc<Type>) -> u32 {
        if t.flags.intersects(TYPE_FLAGS_INSTANTIABLE | TypeFlags::Intersection) {
            let constrained = self
                .get_base_constraint_of_type(t)
                .unwrap_or_else(|| self.unknown_type());
            if !Arc::ptr_eq(&constrained, t) {
                return self.get_typeof_facts(&constrained);
            }
        }
        let nullable = if self.strict_null_checks {
            0
        } else {
            TYPEOF_EQ_NULLABLE
        };
        let flags = t.flags;
        if flags.intersects(TYPE_FLAGS_STRING_LIKE) {
            return TYPEOF_EQ_STRING | (TYPEOF_NE_ALL_EXCEPT_UNDEFINED & !TYPEOF_NE_OBJECT) | nullable;
        }
        if flags.intersects(TYPE_FLAGS_NUMBER_LIKE) {
            return TYPEOF_EQ_NUMBER | (TYPEOF_NE_ALL_EXCEPT_UNDEFINED & !TYPEOF_NE_NUMBER) | nullable;
        }
        if flags.intersects(TYPE_FLAGS_BIG_INT_LIKE) {
            return TYPEOF_EQ_BIGINT | (TYPEOF_NE_ALL_EXCEPT_UNDEFINED & !TYPEOF_NE_BIGINT) | nullable;
        }
        if flags.intersects(TYPE_FLAGS_BOOLEAN_LIKE) {
            return TYPEOF_EQ_BOOLEAN | (TYPEOF_NE_ALL_EXCEPT_UNDEFINED & !TYPEOF_NE_BOOLEAN) | nullable;
        }
        if flags.contains(TypeFlags::Object) {
            if self.is_empty_object_type(t) {
                return TYPEOF_FACTS_ALL & !TYPEOF_EQ_NULLABLE;
            }
            if self.is_function_object_type(t) {
                return TYPEOF_EQ_FUNCTION
                    | TYPEOF_EQ_HOST_OBJECT
                    | TYPEOF_NE_STRING
                    | TYPEOF_NE_NUMBER
                    | TYPEOF_NE_BIGINT
                    | TYPEOF_NE_BOOLEAN
                    | TYPEOF_NE_SYMBOL
                    | TYPEOF_NE_OBJECT
                    | nullable;
            }
            return TYPEOF_EQ_OBJECT
                | TYPEOF_EQ_HOST_OBJECT
                | TYPEOF_NE_STRING
                | TYPEOF_NE_NUMBER
                | TYPEOF_NE_BIGINT
                | TYPEOF_NE_BOOLEAN
                | TYPEOF_NE_SYMBOL
                | TYPEOF_NE_FUNCTION
                | nullable;
        }
        if flags.contains(TypeFlags::Void) {
            return TYPEOF_NE_ALL_EXCEPT_UNDEFINED | TYPEOF_EQ_UNDEFINED;
        }
        if flags.contains(TypeFlags::Undefined) {
            return TYPEOF_NE_ALL_EXCEPT_UNDEFINED | TYPEOF_EQ_UNDEFINED;
        }
        if flags.contains(TypeFlags::Null) {
            return TYPEOF_EQ_OBJECT
                | TYPEOF_EQ_NULL
                | (TYPEOF_NE_ALL_EXCEPT_UNDEFINED & !TYPEOF_NE_OBJECT)
                | TYPEOF_NE_UNDEFINED;
        }
        if flags.intersects(TYPE_FLAGS_ES_SYMBOL_LIKE) {
            return TYPEOF_EQ_SYMBOL | (TYPEOF_NE_ALL_EXCEPT_UNDEFINED & !TYPEOF_NE_SYMBOL) | nullable;
        }
        if flags.contains(TypeFlags::NonPrimitive) {
            return TYPEOF_EQ_OBJECT
                | TYPEOF_EQ_HOST_OBJECT
                | TYPEOF_NE_STRING
                | TYPEOF_NE_NUMBER
                | TYPEOF_NE_BIGINT
                | TYPEOF_NE_BOOLEAN
                | TYPEOF_NE_SYMBOL
                | TYPEOF_NE_FUNCTION
                | nullable;
        }
        if flags.contains(TypeFlags::Never) {
            return 0;
        }
        if flags.contains(TypeFlags::Union)
            && let Some(types) = t.types()
        {
            let mut facts = 0;
            for c in types {
                facts |= self.get_typeof_facts(&Arc::clone(c));
            }
            return facts;
        }
        if flags.contains(TypeFlags::Intersection)
            && let Some(types) = t.types()
        {
            return self.get_typeof_facts_of_intersection(t, types);
        }
        TYPEOF_FACTS_UNKNOWN
    }

    fn get_typeof_facts_of_intersection(
        &mut self,
        t: &Arc<Type>,
        types: &[Arc<Type>],
    ) -> u32 {
        let ignore_objects = self.maybe_type_of_kind(t, TYPE_FLAGS_PRIMITIVE);
        let mut ored = 0;
        let mut anded = TYPEOF_FACTS_ALL;
        for c in types {
            if ignore_objects && c.flags.contains(TypeFlags::Object) {
                continue;
            }
            let f = self.get_typeof_facts(&Arc::clone(c));
            ored |= f;
            anded &= f;
        }
        (ored & TYPEOF_OR_FACTS_MASK) | (anded & TYPEOF_AND_FACTS_MASK)
    }

    pub(crate) fn is_empty_object_type(&self, t: &Arc<Type>) -> bool {
        let Some(structured) = t.as_structured() else {
            return false;
        };
        structured.members.entries.is_empty()
            && structured.signatures.is_empty()
            && structured.index_infos.is_empty()
            && structured.properties.is_empty()
    }

    pub(crate) fn is_function_object_type(&mut self, t: &Arc<Type>) -> bool {
        if t.object_flags.contains(ObjectFlags::EvolvingArray) {
            return false;
        }
        if !self.get_signatures_of_type(t, SignatureKind::Call).is_empty() {
            return true;
        }
        let has_bind = t
            .as_structured()
            .is_some_and(|s| s.members.get("bind").is_some());
        if !has_bind {
            return false;
        }
        match self.global_function_type_of("Function") {
            Some(ft) => self.is_type_subtype_of(t, &ft),
            None => false,
        }
    }

    pub(crate) fn typeof_ne_facts_of_witness(text: &str) -> u32 {
        match text {
            "string" => TYPEOF_NE_STRING,
            "number" => TYPEOF_NE_NUMBER,
            "bigint" => TYPEOF_NE_BIGINT,
            "boolean" => TYPEOF_NE_BOOLEAN,
            "symbol" => TYPEOF_NE_SYMBOL,
            "undefined" => TYPEOF_NE_UNDEFINED,
            "object" => TYPEOF_NE_OBJECT,
            "function" => TYPEOF_NE_FUNCTION,
            _ => TYPEOF_NE_HOST_OBJECT,
        }
    }
}
