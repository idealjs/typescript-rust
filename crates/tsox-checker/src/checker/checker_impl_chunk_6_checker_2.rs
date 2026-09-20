#![allow(unused_imports)]

use crate::checker::checker_impl_chunk_6::*;

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

            // Go getTupleBaseType：元组的属性存在性经 Array<元素并集> 基类型判定
            if self.is_tuple_type(t) {
                return name == "length" || self.global_interface_has_property("Array", name);
            }

            if t.object_flags.contains(ObjectFlags::Anonymous)
                && !structured.signatures.is_empty()
            {
                let fallback_type = if structured.call_signature_count > 0 {
                    self.global_callable_function_type()
                } else {
                    self.global_newable_function_type()
                };
                if let Some(ft) = fallback_type
                    && self.get_property_of_type(&ft, name).is_some()
                {
                    return true;
                }
            }

            if t.flags.contains(TypeFlags::Object)
                && !t.object_flags.contains(ObjectFlags::Reference)
            {
                return self.global_interface_has_property("Object", name);
            }
        }

        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                let types = u.union_or_intersection.types.clone();
                for ct in &types {
                    if ct
                        .flags
                        .intersects(TypeFlags::Undefined | TypeFlags::Null | TypeFlags::Never)
                    {
                        continue;
                    }
                    if !self.constituent_admits_property(ct, name) {
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
            if let Some(constraint) = self.get_constraint_of_type_parameter(t)
                && !constraint.flags.contains(TypeFlags::Unknown)
            {
                return self.has_property_of_type(&constraint, name);
            }
            if self.strict_null_checks {
                return false;
            }
            return self.global_interface_has_property("Object", name);
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
            if let Some(array_sym) = self.globals.get("Array")
                && let Some(declared) = self
                    .type_alias_links
                    .get(array_sym)
                    .and_then(|l| l.declared_type.clone())
                && declared
                    .as_structured()
                    .is_some_and(|s| s.members.get(name).is_some())
            {
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
            return self.global_interface_has_property("String", name)
                || self.global_interface_has_property("Object", name);
        }

        if t.flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
        {
            return self.global_interface_has_property("Number", name)
                || self.global_interface_has_property("Object", name);
        }

        if t.flags
            .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
        {
            return self.global_interface_has_property("Boolean", name)
                || self.global_interface_has_property("Object", name);
        }

        if t.flags
            .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
        {
            return self.global_interface_has_property("BigInt", name)
                || self.global_interface_has_property("Object", name);
        }

        if t.flags.intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol) {
            return self.global_interface_has_property("Symbol", name)
                || self.global_interface_has_property("Object", name);
        }

        if t.flags.contains(TypeFlags::Void) {
            return false;
        }

        if t.flags.contains(TypeFlags::Object | TypeFlags::Enum) {
            return true;
        }

        true
    }

    pub(crate) fn expression_has_side_effects(&self, node: &Arc<Node>) -> bool {
        let mut cur = node;
        while let tsox_frontend::ast::NodeData::ParenthesizedExpression(p) = &cur.data {
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
            tsox_frontend::ast::NodeData::Identifier(_)
            | tsox_frontend::ast::NodeData::StringLiteral(_)
            | tsox_frontend::ast::NodeData::RegularExpressionLiteral(_)
            | tsox_frontend::ast::NodeData::TaggedTemplateExpression(_)
            | tsox_frontend::ast::NodeData::TemplateExpression(_)
            | tsox_frontend::ast::NodeData::NoSubstitutionTemplateLiteral(_)
            | tsox_frontend::ast::NodeData::NumericLiteral(_)
            | tsox_frontend::ast::NodeData::BigIntLiteral(_)
            | tsox_frontend::ast::NodeData::FunctionExpression(_)
            | tsox_frontend::ast::NodeData::ClassExpression(_)
            | tsox_frontend::ast::NodeData::ArrowFunction(_)
            | tsox_frontend::ast::NodeData::ArrayLiteralExpression(_)
            | tsox_frontend::ast::NodeData::ObjectLiteralExpression(_)
            | tsox_frontend::ast::NodeData::TypeOfExpression(_)
            | tsox_frontend::ast::NodeData::NonNullExpression(_)
            | tsox_frontend::ast::NodeData::JsxSelfClosingElement(_)
            | tsox_frontend::ast::NodeData::JsxElement(_) => false,
            tsox_frontend::ast::NodeData::ConditionalExpression(c) => {
                self.expression_has_side_effects(&c.when_true)
                    || self.expression_has_side_effects(&c.when_false)
            }
            tsox_frontend::ast::NodeData::BinaryExpression(b) => {
                Self::is_assignment_operator(b.operator_token.kind)
                    || self.expression_has_side_effects(&b.left)
                    || self.expression_has_side_effects(&b.right)
            }
            tsox_frontend::ast::NodeData::PrefixUnaryExpression(p) => !matches!(
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
        let Some(paren) = comma.parent() else {
            return false;
        };
        if paren.kind != SyntaxKind::ParenthesizedExpression {
            return false;
        }
        let tsox_frontend::ast::NodeData::BinaryExpression(b) = &comma.data else {
            return false;
        };
        let zero_left = matches!(&b.left.data, tsox_frontend::ast::NodeData::NumericLiteral(n) if n.text == "0");
        if !zero_left {
            return false;
        }
        let Some(grand) = paren.parent() else {
            return false;
        };
        let call_uses_paren = matches!(&grand.data, tsox_frontend::ast::NodeData::CallExpression(ce)
            if Arc::as_ptr(&ce.expression) == Arc::as_ptr(&paren));
        if !call_uses_paren && grand.kind != SyntaxKind::TaggedTemplateExpression {
            return false;
        }
        match &b.right.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(_)
            | tsox_frontend::ast::NodeData::ElementAccessExpression(_) => true,
            tsox_frontend::ast::NodeData::Identifier(id) => id.text == "eval",
            _ => false,
        }
    }

    pub(crate) fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }
}
