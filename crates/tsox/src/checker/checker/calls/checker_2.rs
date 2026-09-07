#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn report_invocation_error(
        &mut self,
        callee_expr: &Arc<Node>,
        callee_type: &Arc<Type>,
        is_new: bool,
    ) {
        let head = if is_new {
            THIS_EXPRESSION_IS_NOT_CONSTRUCTABLE
        } else {
            THIS_EXPRESSION_IS_NOT_CALLABLE
        };
        let no_sigs = if is_new {
            TYPE_0_HAS_NO_CONSTRUCT_SIGNATURES
        } else {
            TYPE_0_HAS_NO_CALL_SIGNATURES
        };
        let chain = if callee_type.flags.contains(TypeFlags::Union)
            && let Some(u) = callee_type.as_union_or_intersection()
        {
            let union_str = self.type_to_string(callee_type);
            let mut has_signatures = false;
            let mut first_without: Option<String> = None;
            for c in u.types.iter() {
                let n = if is_new {
                    c.as_structured()
                        .map(|s| s.construct_signatures().len())
                        .unwrap_or(0)
                } else {
                    c.as_structured()
                        .map(|s| s.call_signatures().len())
                        .unwrap_or(0)
                };
                if n != 0 {
                    has_signatures = true;
                    if first_without.is_some() {
                        break;
                    }
                } else if first_without.is_none() {
                    first_without = Some(self.type_to_string(c));
                }
            }
            let msg = if !has_signatures {
                if is_new {
                    NO_CONSTITUENT_OF_TYPE_0_IS_CONSTRUCTABLE
                } else {
                    NO_CONSTITUENT_OF_TYPE_0_IS_CALLABLE
                }
            } else if first_without.is_some() {
                if is_new {
                    NOT_ALL_CONSTITUENTS_OF_TYPE_0_ARE_CONSTRUCTABLE
                } else {
                    NOT_ALL_CONSTITUENTS_OF_TYPE_0_ARE_CALLABLE
                }
            } else if is_new {
                EACH_MEMBER_OF_THE_UNION_TYPE_0_HAS_CONSTRUCT_SIGNATURES_BUT_NONE_OF_THOSE_SIGNATURES_ARE_COMPATIBLE_WITH_EACH_OTHER
            } else {
                EACH_MEMBER_OF_THE_UNION_TYPE_0_HAS_SIGNATURES_BUT_NONE_OF_THOSE_SIGNATURES_ARE_COMPATIBLE_WITH_EACH_OTHER
            };
            let mut outer = crate::ast::Diagnostic::new(
                self.current_file.clone(),
                callee_expr.loc,
                msg,
                vec![union_str],
            );
            if let Some(first) = first_without.filter(|_| has_signatures) {
                outer.message_chain = vec![crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    callee_expr.loc,
                    no_sigs,
                    vec![first],
                )];
            }
            vec![outer]
        } else {
            let apparent_str = if callee_type.flags.contains(TypeFlags::Intersection)
                && self.is_never_intersection(callee_type)
            {
                "never".to_string()
            } else {
                match self.primitive_apparent_name(callee_type) {
                    Some(name) => name.to_string(),
                    None => self.type_to_string(callee_type),
                }
            };
            vec![crate::ast::Diagnostic::new(
                self.current_file.clone(),
                callee_expr.loc,
                no_sigs,
                vec![apparent_str],
            )]
        };
        let mut diag =
            crate::ast::Diagnostic::new(self.current_file.clone(), callee_expr.loc, head, vec![]);
        diag.message_chain = chain;
        self.diagnostics.add(diag);
    }

    pub(crate) fn primitive_apparent_name(&self, t: &Arc<Type>) -> Option<&'static str> {
        let name = if t.flags.intersects(
            TypeFlags::String
                | TypeFlags::StringLiteral
                | TypeFlags::TemplateLiteral
                | TypeFlags::StringMapping,
        ) {
            "String"
        } else if t
            .flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
        {
            "Number"
        } else if t
            .flags
            .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
        {
            "Boolean"
        } else if t
            .flags
            .intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol)
        {
            "Symbol"
        } else if t
            .flags
            .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
        {
            "BigInt"
        } else {
            return None;
        };
        self.globals.get(name).map(|_| name)
    }

    pub(crate) fn is_never_intersection(&mut self, t: &Arc<Type>) -> bool {
        let Some(ui) = t.as_union_or_intersection() else {
            return false;
        };
        let domain = |t: &Arc<Type>| -> u8 {
            if t.flags.intersects(
                TypeFlags::String
                    | TypeFlags::StringLiteral
                    | TypeFlags::TemplateLiteral
                    | TypeFlags::StringMapping,
            ) {
                1
            } else if t
                .flags
                .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
            {
                2
            } else if t
                .flags
                .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
            {
                3
            } else if t
                .flags
                .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
            {
                4
            } else if t
                .flags
                .intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol)
            {
                5
            } else if t.flags.contains(TypeFlags::Undefined) {
                6
            } else if t.flags.contains(TypeFlags::Null) {
                7
            } else {
                0
            }
        };
        let disjoint = |a: &Arc<Type>, b: &Arc<Type>| -> bool {
            let (da, db) = (domain(a), domain(b));
            if da == 0 || db == 0 {
                return false;
            }
            if da != db {
                return true;
            }
            match (a.literal_value(), b.literal_value()) {
                (Some(x), Some(y)) => x != y,
                _ => false,
            }
        };
        for (i, c) in ui.types.iter().enumerate() {
            let Some(cs) = c.as_structured() else {
                continue;
            };
            for prop in &cs.properties {
                for (j, other) in ui.types.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let Some(os) = other.as_structured() else {
                        continue;
                    };
                    if let Some(other_prop) =
                        os.properties.iter().find(|p| p.name == prop.name).cloned()
                    {
                        let pt = self.get_type_of_symbol(prop);
                        let ot = self.get_type_of_symbol(&other_prop);
                        if disjoint(&pt, &ot) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}
