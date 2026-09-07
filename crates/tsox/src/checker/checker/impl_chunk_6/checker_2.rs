#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn has_property_of_type(&mut self, t: &Arc<Type>, name: &str) -> bool {
        if t.flags.contains(TypeFlags::IndexedAccess)
            && let Some(constraint) = self.constraint_of_indexed_access(t)
        {
            return self.has_property_of_type(&constraint, name);
        }

        if t.flags.intersects(
            TypeFlags::Any
                | TypeFlags::Unknown
                | TypeFlags::Never
                | TypeFlags::Undefined
                | TypeFlags::Null,
        ) {
            return true;
        }

        if let Some(structured) = t.as_structured() {
            if structured.members.get(name).is_some() {
                return true;
            }

            if matches!(&t.data, TypeData::Mapped(m) if m.type_parameter.is_some()) {
                return true;
            }
            if !structured.index_infos.is_empty() {
                return true;
            }

            if t.object_flags.contains(ObjectFlags::EvolvingArray) {
                return name == "length" || self.is_array_mutation_method(name);
            }

            if t.object_flags.contains(ObjectFlags::Anonymous)
                && structured.call_signature_count > 0
                && self.global_interface_has_property("Function", name)
            {
                return true;
            }

            if t.flags.contains(TypeFlags::Object)
                && !t.object_flags.contains(ObjectFlags::Reference)
            {
                return self.global_interface_has_property("Object", name);
            }
        }

        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                for ct in &u.union_or_intersection.types {
                    if ct.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
                        continue;
                    }
                    if !self.has_property_of_type(ct, name) {
                        return false;
                    }
                }
                return true;
            }
        }

        if t.flags.contains(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = &t.data {
                for ct in &i.union_or_intersection.types {
                    if self.has_property_of_type(ct, name) {
                        return true;
                    }
                }
                return false;
            }
        }

        if t.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(t) {
                return self.has_property_of_type(&constraint, name);
            }

            return true;
        }

        if t.flags.contains(TypeFlags::Conditional) {
            if let Some(constraint) = self.constraint_of_conditional_type(t) {
                return self.has_property_of_type(&constraint, name);
            }

            return true;
        }

        if t.flags.contains(TypeFlags::IndexedAccess) {
            if let TypeData::IndexedAccess(ia) = &t.data {
                if let (Some(o), Some(i)) = (&ia.object_type, &ia.index_type) {
                    let obj = self.get_base_constraint_or_type(o);
                    let idx = self.get_base_constraint_or_type(i);
                    if !self.type_flags_is_generic_object_type(&obj)
                        && !self.type_flags_is_generic_index_type(&idx)
                    {
                        let resolved = self.get_indexed_access_type(&obj, &idx);
                        return self.has_property_of_type(&resolved, name);
                    }
                }
            }
            return true;
        }

        if self.is_array_type(t) {
            if name == "length" {
                return true;
            }
            if (self.is_auto_array_type(t) || t.object_flags.contains(ObjectFlags::EvolvingArray))
                && self.is_array_mutation_method(name)
            {
                return true;
            }

            if self.global_interface_has_property("Array", name) {
                return true;
            }
            return false;
        }

        if t.object_flags.contains(ObjectFlags::EvolvingArray) {
            return name == "length" || self.is_array_mutation_method(name);
        }

        if self.is_tuple_type(t) {
            return name == "length";
        }

        if t.flags
            .intersects(TypeFlags::String | TypeFlags::StringLiteral)
        {
            return self.global_interface_has_property("String", name);
        }

        if t.flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
        {
            return self.global_interface_has_property("Number", name);
        }

        if t.flags
            .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
        {
            return self.global_interface_has_property("Boolean", name);
        }

        if t.flags
            .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
        {
            return self.global_interface_has_property("BigInt", name);
        }

        if t.flags
            .intersects(TypeFlags::ESSymbol | TypeFlags::Void | TypeFlags::UniqueESSymbol)
        {
            return false;
        }

        if t.flags.contains(TypeFlags::Object | TypeFlags::Enum) {
            return true;
        }

        true
    }

    pub(crate) fn expression_has_side_effects(&self, node: &Arc<Node>) -> bool {
        let mut cur = node;
        while let crate::ast::NodeData::ParenthesizedExpression(p) = &cur.data {
            cur = &p.expression;
        }

        if matches!(
            cur.kind,
            SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::NullKeyword
                | SyntaxKind::UndefinedKeyword
        ) {
            return false;
        }
        match &cur.data {
            crate::ast::NodeData::Identifier(_)
            | crate::ast::NodeData::StringLiteral(_)
            | crate::ast::NodeData::RegularExpressionLiteral(_)
            | crate::ast::NodeData::TaggedTemplateExpression(_)
            | crate::ast::NodeData::TemplateExpression(_)
            | crate::ast::NodeData::NoSubstitutionTemplateLiteral(_)
            | crate::ast::NodeData::NumericLiteral(_)
            | crate::ast::NodeData::BigIntLiteral(_)
            | crate::ast::NodeData::FunctionExpression(_)
            | crate::ast::NodeData::ClassExpression(_)
            | crate::ast::NodeData::ArrowFunction(_)
            | crate::ast::NodeData::ArrayLiteralExpression(_)
            | crate::ast::NodeData::ObjectLiteralExpression(_)
            | crate::ast::NodeData::TypeOfExpression(_)
            | crate::ast::NodeData::NonNullExpression(_)
            | crate::ast::NodeData::JsxSelfClosingElement(_)
            | crate::ast::NodeData::JsxElement(_) => false,
            crate::ast::NodeData::ConditionalExpression(c) => {
                self.expression_has_side_effects(&c.when_true)
                    || self.expression_has_side_effects(&c.when_false)
            }
            crate::ast::NodeData::BinaryExpression(b) => {
                Self::is_assignment_operator(b.operator_token.kind)
                    || self.expression_has_side_effects(&b.left)
                    || self.expression_has_side_effects(&b.right)
            }
            crate::ast::NodeData::PrefixUnaryExpression(p) => !matches!(
                p.operator,
                SyntaxKind::ExclamationToken
                    | SyntaxKind::PlusToken
                    | SyntaxKind::MinusToken
                    | SyntaxKind::TildeToken
            ),
            _ => true,
        }
    }

    pub(crate) fn is_indirect_call_comma(&self, comma: &Arc<Node>) -> bool {
        let Some(paren) = comma.parent.as_ref() else {
            return false;
        };
        if paren.kind != SyntaxKind::ParenthesizedExpression {
            return false;
        }
        let crate::ast::NodeData::BinaryExpression(b) = &comma.data else {
            return false;
        };
        let zero_left =
            matches!(&b.left.data, crate::ast::NodeData::NumericLiteral(n) if n.text == "0");
        if !zero_left {
            return false;
        }
        let Some(grand) = paren.parent.as_ref() else {
            return false;
        };
        let call_uses_paren = matches!(&grand.data, crate::ast::NodeData::CallExpression(ce)
            if std::ptr::eq(&ce.expression, paren));
        if !call_uses_paren && grand.kind != SyntaxKind::TaggedTemplateExpression {
            return false;
        }
        match &b.right.data {
            crate::ast::NodeData::PropertyAccessExpression(_)
            | crate::ast::NodeData::ElementAccessExpression(_) => true,
            crate::ast::NodeData::Identifier(id) => id.text == "eval",
            _ => false,
        }
    }

    pub(crate) fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }
}
