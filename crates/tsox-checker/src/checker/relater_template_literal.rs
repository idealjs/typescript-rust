#![allow(unused_imports)]

use crate::checker::inference::*;

impl Checker {
    pub fn is_type_matched_by_template_literal_type(
        &mut self,
        source: &Arc<Type>,
        target: &TemplateLiteralTypeData,
    ) -> bool {
        let Some(inferences) = self.infer_types_from_template_literal_type(source, target) else {
            return false;
        };
        for (i, inference) in inferences.iter().enumerate() {
            if !self.is_valid_type_for_template_literal_placeholder(inference, &target.types[i]) {
                return false;
            }
        }
        true
    }

    pub fn is_valid_type_for_template_literal_placeholder(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> bool {
        if target.flags.contains(TypeFlags::Intersection) {
            return target.types().is_some_and(|ts| {
                ts.iter().all(|t| {
                    self.is_empty_object_type(t)
                        || self.is_valid_type_for_template_literal_placeholder(source, t)
                })
            });
        }
        if target.flags.contains(TypeFlags::String) {
            return true;
        }
        let was_silent = self.silence_relation_chain();
        let related = self.is_type_assignable_to(source, target);
        self.restore_relation_chain(was_silent);
        if related {
            return true;
        }
        if source.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &source.data
                && let LiteralValue::String(value) = &lit.value
            {
                if target.flags.contains(TypeFlags::Number)
                    && crate::checker::utilities_get_assignment_target::is_valid_number_string(
                        value, false,
                    )
                {
                    return true;
                }
                if target.flags.contains(TypeFlags::BigInt)
                    && crate::checker::utilities_get_assignment_target::is_valid_big_int_string(
                        value, false,
                    )
                {
                    return true;
                }
                if target.flags.intersects(TypeFlags::BooleanLiteral | TYPE_FLAGS_NULLABLE)
                    && target.intrinsic_name() == Some(value.as_str())
                {
                    return true;
                }
                if target.flags.contains(TypeFlags::StringMapping)
                    && self.is_member_of_string_mapping(source, target)
                {
                    return true;
                }
                if target.flags.contains(TypeFlags::TemplateLiteral)
                    && let TypeData::TemplateLiteral(tl) = &target.data
                    && self.is_type_matched_by_template_literal_type(source, tl)
                {
                    return true;
                }
            }
        }
        if source.flags.contains(TypeFlags::TemplateLiteral)
            && let TypeData::TemplateLiteral(sl) = &source.data
            && sl.texts.len() == 2
            && sl.texts[0].is_empty()
            && sl.texts[1].is_empty()
        {
            let was_silent = self.silence_relation_chain();
            let related = self.is_type_assignable_to(&sl.types[0], target);
            self.restore_relation_chain(was_silent);
            return related;
        }
        false
    }
}
