use std::sync::Arc;

use tsox_frontend::ast::{Node, Symbol, SyntaxKind};

use crate::checker::checker::Checker;
use crate::checker::types::{ObjectFlags, Type, TypeFlags, UnionReduction};

pub fn is_entity_name_expression_local(node: &Node) -> bool {
    node.kind == SyntaxKind::Identifier
        || (node.kind == SyntaxKind::PropertyAccessExpression
            && node
                .expression()
                .is_some_and(|e| is_entity_name_expression_local(e.as_ref())))
}

impl Checker {
    pub fn get_literal_type_from_properties(
        &mut self,
        t: &Arc<Type>,
        include: TypeFlags,
        include_origin: bool,
    ) -> Arc<Type> {
        // 现行 get_union_type_ex 无 origin 形参(r24k7 交接同类缺口),Go origin 实参无处挂载,留交接
        let _ = include_origin;
        let props = self.get_properties_of_type(t);
        let index_infos = self.get_index_infos_of_type(t);
        let mut types: Vec<Arc<Type>> = Vec::with_capacity(props.len() + index_infos.len());
        for prop in &props {
            types.push(self.get_literal_type_from_property_ex(prop, include, false));
        }
        let enum_number_index_info = self.enum_number_index_info();
        let string_type = self.string_type();
        for info in &index_infos {
            if let Some(key_type) = info.key_type.as_ref()
                && !Arc::ptr_eq(info, &enum_number_index_info)
                && self.is_key_type_included(key_type, include)
            {
                if Arc::ptr_eq(key_type, &string_type) && include.intersects(TypeFlags::NUMBER) {
                    types.push(self.string_or_number_type());
                } else {
                    types.push(Arc::clone(key_type));
                }
            }
        }
        self.get_union_type_ex(types, UnionReduction::Literal)
    }

    pub fn get_literal_type_from_property_ex(
        &mut self,
        prop: &Arc<Symbol>,
        include: TypeFlags,
        include_non_public: bool,
    ) -> Arc<Type> {
        if include_non_public
            || !crate::checker::exports::get_declaration_modifier_flags_from_symbol(prop)
                .intersects(tsox_frontend::ast::ModifierFlags::NonPublicAccessibilityModifier)
        {
            let late_bound = self.get_late_bound_symbol(prop);
            let mut t = self
                .value_symbol_links
                .get_or_default(&late_bound)
                .name_type
                .clone();
            if t.is_none() {
                if prop.name == tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT {
                    t = Some(self.get_string_literal_type("default"));
                } else {
                    if let Some(decl) = prop.value_declaration.as_ref()
                        && let Some(name) = tsox_frontend::ast::utilities::get_name_of_declaration(decl)
                    {
                        t = self.get_literal_type_from_property_name(&name);
                    }
                    if t.is_none() && !crate::checker::utilities_has_only_expression_initialization::is_known_symbol(prop) {
                        t = Some(self.get_string_literal_type(&prop.name));
                    }
                }
            }
            if let Some(t) = t
                && t.flags.intersects(include)
            {
                return t;
            }
        }
        self.never_type()
    }
}
