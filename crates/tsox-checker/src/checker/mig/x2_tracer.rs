#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::checker::Checker;
use crate::checker::types::{ConditionalTypeData, ObjectFlags, Type, TypeFlags};
use crate::checker::utilities_token_is_identifier_or_keyword::type_to_string;

use super::x3::{wrap_type, TracedType, TracedTypeAdapter};

impl TracedTypeAdapter {
    pub(crate) fn conditional_check_type(&self) -> Option<TracedType> {
        let t = unsafe { &*self.t };
        if !t.flags.intersects(TypeFlags::Conditional) {
            return None;
        }
        t.as_conditional_type()?
            .check_type
            .as_deref()
            .map(wrap_type)
    }

    pub(crate) fn conditional_extends_type(&self) -> Option<TracedType> {
        let t = unsafe { &*self.t };
        if !t.flags.intersects(TypeFlags::Conditional) {
            return None;
        }
        t.as_conditional_type()?
            .extends_type
            .as_deref()
            .map(wrap_type)
    }

    pub(crate) fn conditional_true_type(&self) -> Option<TracedType> {
        let t = unsafe { &*self.t };
        if !t.flags.intersects(TypeFlags::Conditional) {
            return None;
        }
        t.as_conditional_type()?
            .resolved_true_type
            .get()
            .map(|t| wrap_type(t))
    }

    pub(crate) fn conditional_false_type(&self) -> Option<TracedType> {
        let t = unsafe { &*self.t };
        if !t.flags.intersects(TypeFlags::Conditional) {
            return None;
        }
        t.as_conditional_type()?
            .resolved_false_type
            .get()
            .map(|t| wrap_type(t))
    }
}

impl Checker {
    pub(crate) fn traced_type_display(&self, a: &TracedTypeAdapter) -> String {
        let t = unsafe { &*a.t };
        if t.object_flags.intersects(ObjectFlags::Anonymous)
            || t.flags.intersects(
                TypeFlags::StringLiteral
                    | TypeFlags::NumberLiteral
                    | TypeFlags::BigIntLiteral
                    | TypeFlags::BooleanLiteral
                    | TypeFlags::TemplateLiteral
                    | TypeFlags::Union
                    | TypeFlags::Intersection,
            )
        {
            return type_to_string(t);
        }
        String::new()
    }
}
