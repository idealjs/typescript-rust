#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;
use crate::checker::relater_relation::{RelationKind, RECURSION_FLAGS_BOTH};

use crate::checker::checker_checker_checker::Checker;
use crate::checker::nodecopy_builder::NodeBuilderImpl;
use crate::checker::*;

pub(crate) type Relater = Checker;
use tsox_frontend::ast::{self, *};
use tsox_frontend::ast::is_computed_property_name;
use tsox_frontend::ast::is_entity_name_expression;

impl Relater {
    pub fn is_property_symbol_type_related(
        &mut self,
        source_prop: &Arc<Symbol>,
        target_prop: &Arc<Symbol>,
        get_type_of_source_property: &mut dyn FnMut(&Arc<Symbol>) -> Arc<Type>,
        report_errors: bool,
        intersection_state: IntersectionState,
        relation: RelationKind,
    ) -> Ternary {
        let target_is_optional =
            self.strict_null_checks && target_prop.check_flags.contains(CheckFlags::ReadPartial | CheckFlags::WritePartial);
        let target_type = self.get_non_missing_type_of_symbol(target_prop);
        let effective_target = self.add_optionality_ex(&target_type, false, target_is_optional);
        let any_flags = if relation == RelationKind::StrictSubtype {
            TypeFlags::Any
        } else {
            TypeFlags::ANY_OR_UNKNOWN
        };
        if effective_target.flags.intersects(any_flags) {
            return Ternary::True;
        }
        let effective_source = get_type_of_source_property(source_prop);
        self.is_related_to_ex(
            &effective_source,
            &effective_target,
            RECURSION_FLAGS_BOTH,
            report_errors,
            None,
            intersection_state,
        )
    }

    pub fn is_related_to_ex(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        recursion_flags: RecursionFlags,
        report_errors: bool,
        head_message: Option<&tsox_core::diagnostics::Message>,
        intersection_state: IntersectionState,
    ) -> Ternary {
        let _ = (recursion_flags, report_errors, head_message, intersection_state);
        if self.is_type_related_to(source, target, RelationKind::Assignable) {
            Ternary::True
        } else {
            Ternary::False
        }
    }
}

impl Checker {
    pub fn is_type_presence_possible(
        &mut self,
        t: &Arc<Type>,
        prop_name: &str,
        assume_true: bool,
    ) -> bool {
        if let Some(prop) = self.get_property_of_type(t, prop_name) {
            return prop.flags.contains(SymbolFlags::Optional)
                || prop.check_flags.contains(CheckFlags::ReadPartial | CheckFlags::WritePartial)
                || assume_true;
        }
        self.get_applicable_index_info_for_name(t, prop_name).is_some() || !assume_true
    }
}

impl<'a> NodeBuilderImpl<'a> {
    // 单线程 checker 内经共享引用回写缓存，与 BUILDER_CHECKER 通道/w4a checker_ptr 同款约定
    #[allow(invalid_reference_casting)]
    pub fn is_trivially_serializable_computed_name(&self, e: Option<&Arc<Node>>) -> bool {
        let Some(e) = e else {
            return false;
        };
        let shape_good = e.name().is_some_and(|name| {
            is_computed_property_name(&name)
                && name.expression().is_some_and(|expr| is_entity_name_expression(&expr))
        });
        if !shape_good {
            return false;
        }
        let name_expr = e.name().unwrap().expression().unwrap();
        let ch = unsafe { &mut *(self.ch as *const Checker as *mut Checker) };
        ch.is_entity_name_visible(
            &name_expr,
            self.ctx.borrow().enclosing_declaration.as_ref().unwrap(),
        )
        .accessibility
            == SymbolAccessibility::Accessible
    }
}
