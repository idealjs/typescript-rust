#![allow(unused_imports)]

use crate::checker::checker_calls::*;

impl Checker {
    pub(crate) fn report_invocation_error(
        &mut self,
        callee_expr: &Arc<Node>,
        callee_type: &Arc<Type>,
        is_new: bool,
    ) {
        let apparent = self.get_apparent_type(callee_type);
        let kind = if is_new {
            SignatureKind::Construct
        } else {
            SignatureKind::Call
        };
        self.invocation_error(callee_expr, &apparent, kind, None);
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
