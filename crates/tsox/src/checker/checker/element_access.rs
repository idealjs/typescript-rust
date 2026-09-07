use std::sync::Arc;

use crate::ast::Node;

use super::*;

impl Checker {
    pub(crate) fn get_type_of_element_access(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (obj_expr, arg_expr) = match &node.data {
            crate::ast::NodeData::ElementAccessExpression(data) => {
                (&data.expression, &data.argument_expression)
            }
            _ => return self.get_any_type(),
        };

        {
            let arg_type = self.get_type_of_node(arg_expr);

            let is_type_param_or_union_of = arg_type.is_type_parameter()
                || (arg_type.is_union()
                    && arg_type
                        .types()
                        .is_some_and(|ts| ts.iter().all(|t| t.is_type_parameter())));
            if !arg_type.flags.intersects(TypeFlags::Any | TypeFlags::Never)
                && !is_type_param_or_union_of
            {
                let parts: Vec<Arc<Type>> = if arg_type.is_union() {
                    arg_type.types().map(|ts| ts.to_vec()).unwrap_or_default()
                } else {
                    vec![Arc::clone(&arg_type)]
                };
                for p in parts {
                    if p.flags.intersects(
                        TypeFlags::Any
                            | TypeFlags::Never
                            | TypeFlags::String
                            | TypeFlags::StringLiteral
                            | TypeFlags::Number
                            | TypeFlags::NumberLiteral
                            | TypeFlags::ESSymbol
                            | TypeFlags::EnumLiteral
                            | TypeFlags::StringMapping,
                    ) {
                        continue;
                    }
                    let type_str = self.type_to_string(&p);
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        arg_expr.loc,
                        crate::diagnostics::messages_generated::
                            TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                        vec![type_str],
                    ));
                }
            }
        }
        let obj_type = self.get_type_of_node(obj_expr);

        if obj_type.flags.contains(TypeFlags::Union)
            && let Some(members) = obj_type.types().map(|ts| ts.to_vec())
        {
            let mut elem_types: Vec<Arc<Type>> = Vec::new();
            for m in &members {
                if m.flags.contains(TypeFlags::Any) {
                    continue;
                }
                let t = self.element_access_result_type(node, m, arg_expr);
                if !t.flags.contains(TypeFlags::Any) {
                    elem_types.push(t);
                }
            }
            if !elem_types.is_empty() {
                return self.get_union_type(elem_types);
            }
            return self.get_any_type();
        }
        self.element_access_result_type(node, &obj_type, arg_expr)
    }

    fn element_access_result_type(
        &mut self,
        node: &Arc<Node>,
        obj_type: &Arc<Type>,
        arg_expr: &Arc<Node>,
    ) -> Arc<Type> {
        if self.is_tuple_type(obj_type) {
            if let Some(index) = self.get_constant_numeric_value(arg_expr) {
                if let Some(t) = self.get_tuple_element_type(obj_type, index as usize) {
                    return t;
                }
            }

            return self.get_any_type();
        }

        if self.is_array_type(obj_type) {
            return self.get_array_element_type(obj_type);
        }

        if let Some(member_name) = self.literal_element_access_name(arg_expr) {
            if let Some(sym) = self.get_property_of_type(obj_type, &member_name) {
                if let Some(substituted) = self.instantiate_array_member_type(obj_type, &sym) {
                    return self.flow_type_of_access_expression(node, Some(&sym), substituted);
                }
                let prop_type = self.get_type_of_symbol(&sym);
                return self.flow_type_of_access_expression(node, Some(&sym), prop_type);
            }
        }

        if let Some(structured) = obj_type.as_structured() {
            for info in &structured.index_infos {
                if let Some(key_type) = &info.key_type {
                    if key_type.flags.contains(crate::checker::TypeFlags::String)
                        || key_type.flags.contains(crate::checker::TypeFlags::Number)
                    {
                        if let Some(val_type) = &info.value_type {
                            let val_type = Arc::clone(val_type);
                            return self.flow_type_of_access_expression(node, None, val_type);
                        }
                    }
                }
            }
        }

        self.get_any_type()
    }

    fn literal_element_access_name(&self, arg: &Arc<Node>) -> Option<String> {
        match &arg.data {
            crate::ast::NodeData::StringLiteral(data) => Some(data.text.clone()),
            crate::ast::NodeData::NumericLiteral(data) => Some(data.text.clone()),
            _ => None,
        }
    }
}
