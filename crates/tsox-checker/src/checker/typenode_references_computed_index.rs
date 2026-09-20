#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    /// Go getIndexInfosOfIndexSymbol/lateBindIndexSignature：计算名成员名字
    /// 类型非字面量/unique symbol 但可赋给 string|number|symbol 时，为容器
    /// 合成对应键的索引签名；值型取该成员自身的类型（Go 为全部同键兄弟
    /// 属性类型并集，此处取首成员简化）
    pub(crate) fn implied_index_info_of_computed_member(
        &mut self,
        member: &Arc<Node>,
        existing: &[Arc<crate::checker::IndexInfo>],
    ) -> Option<crate::checker::IndexInfo> {
        let name = member.name()?;
        let NodeData::ComputedPropertyName(cd) = &name.data else {
            return None;
        };
        if !tsox_frontend::ast::is_entity_name_expression(&cd.expression) {
            return None;
        }
        let t = self.get_type_of_node(&cd.expression);
        if crate::checker::utilities_token_is_identifier_or_keyword::is_type_usable_as_property_name(&t)
        {
            return None;
        }
        let key = if self.is_type_assignable_to(&t, &self.number_type()) {
            self.number_type()
        } else if self.is_type_assignable_to(&t, &self.es_symbol_type()) {
            self.es_symbol_type()
        } else if self.is_type_assignable_to(&t, &self.string_type()) {
            self.string_type()
        } else {
            return None;
        };
        if existing.iter().any(|i| {
            i.key_type
                .as_ref()
                .is_some_and(|k| k.flags == key.flags)
        }) {
            return None;
        }
        let is_readonly = member
            .modifiers()
            .as_ref()
            .is_some_and(|m| m.modifier_flags.contains(ModifierFlags::Readonly));
        let value = self.computed_member_index_value_type(member);
        Some(crate::checker::IndexInfo {
            key_type: Some(key),
            value_type: Some(value),
            is_readonly,
            declaration: None,
            index_symbol: None,
            components: vec![Arc::clone(member)],
        })
    }

    fn computed_member_index_value_type(&mut self, member: &Arc<Node>) -> Arc<Type> {
        match &member.data {
            NodeData::MethodDeclaration(d) => {
                self.push_scope(member);
                let return_type = match d.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &d.parameters,
                    return_type,
                    false,
                    None,
                    Some(Arc::clone(member)),
                );
                self.pop_scope();
                self.create_function_or_constructor_type(vec![sig], false)
            }
            NodeData::MethodSignatureDeclaration(d) => {
                self.push_scope(member);
                let return_type = match d.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &d.parameters,
                    return_type,
                    false,
                    None,
                    Some(Arc::clone(member)),
                );
                self.pop_scope();
                self.create_function_or_constructor_type(vec![sig], false)
            }
            NodeData::PropertyDeclaration(d) => match d.type_node.as_ref() {
                Some(tn) => self.get_type_from_type_node(tn),
                None => match d.initializer.as_ref() {
                    Some(init) => self.get_widened_type_of_expression(init),
                    None => self.get_any_type(),
                },
            },
            NodeData::PropertySignatureDeclaration(d) => self.get_type_from_type_node(&d.type_node),
            NodeData::GetAccessorDeclaration(d) => match d.type_node.as_ref() {
                Some(tn) => self.get_type_from_type_node(tn),
                None => self.get_any_type(),
            },
            NodeData::SetAccessorDeclaration(d) => {
                let Some(param) = d.parameters.iter().next() else {
                    return self.get_any_type();
                };
                let NodeData::ParameterDeclaration(pd) = &param.data else {
                    return self.get_any_type();
                };
                match pd.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                }
            }
            _ => self.get_any_type(),
        }
    }
}
