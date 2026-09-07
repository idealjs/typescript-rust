#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn property_chain_name(head: &str, tail: &str) -> String {
        fn get_property_name_arg(arg: &str) -> String {
            if let Some(first) = arg.chars().next()
                && matches!(first, '"' | '\'' | '`')
            {
                format!("[{}]", arg)
            } else {
                arg.to_string()
            }
        }
        let head = get_property_name_arg(head);
        let tail = get_property_name_arg(tail);
        let mut head = head;
        if head.starts_with("new ") {
            head = format!("({})", head);
        }
        let mut pos = 0;
        let bytes = tail.as_bytes();
        loop {
            if tail[pos..].starts_with('(') {
                pos += 1;
            } else if tail[pos..].starts_with("new ") {
                pos += 4;
            } else {
                break;
            }
        }
        let _ = bytes;
        let suffix = &tail[pos..];
        let prefix = &tail[..pos];
        if suffix.starts_with('[') {
            format!("{}{}{}", prefix, head, suffix)
        } else {
            format!("{}{}.{}", prefix, head, suffix)
        }
    }

    pub(crate) fn try_elaborate_primitive_and_object(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        use crate::diagnostics::messages_generated as msg;
        if !source.flags.contains(TypeFlags::Object)
            || !target.flags.intersects(
                TypeFlags::String | TypeFlags::Number | TypeFlags::Boolean | TypeFlags::ESSymbol,
            )
        {
            return;
        }
        let Some(sym) = source.symbol.as_ref() else {
            return;
        };
        let name = match sym.name.as_str() {
            "String" | "Number" | "Boolean" | "Symbol" => sym.name.as_str(),
            _ => return,
        };

        if self.globals.get(name).is_none() {
            return;
        }
        let matches = match name {
            "String" => target.flags.contains(TypeFlags::String),
            "Number" => target.flags.contains(TypeFlags::Number),
            "Boolean" => target.flags.contains(TypeFlags::Boolean),
            _ => target.flags.contains(TypeFlags::ESSymbol),
        };
        if !matches {
            return;
        }
        let target_str = self.type_to_string(target);
        let source_str = self.type_to_string(source);
        self.relater_report_error(
            msg::X_0_IS_A_PRIMITIVE_BUT_1_IS_A_WRAPPER_OBJECT_PREFER_USING_0_WHEN_POSSIBLE,
            vec![target_str, source_str],
        );
    }

    pub(crate) fn relater_report_error(
        &mut self,
        message: crate::diagnostics::Message,
        args: Vec<String>,
    ) {
        use crate::diagnostics::messages_generated as msg;
        if !self.relater_chain_active {
            return;
        }
        if message.key == msg::TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE.key {
            if let Some(top) = self.chain_message_key(0)
                && (top == msg::OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1.key
                    || top == msg::OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_BUT_0_DOES_NOT_EXIST_IN_TYPE_1_DID_YOU_MEAN_TO_WRITE_2.key)
            {
                return;
            }

            let marker = self.chain_message_key(1).map(str::to_string);
            if let Some(m1) = marker {
                let arg = if m1 == msg::CALL_SIGNATURES_WITH_NO_ARGUMENTS_HAVE_INCOMPATIBLE_RETURN_TYPES_0_AND_1.key {
                    Some(format!("{}()", args[0]))
                } else if m1 == msg::CONSTRUCT_SIGNATURES_WITH_NO_ARGUMENTS_HAVE_INCOMPATIBLE_RETURN_TYPES_0_AND_1.key {
                    Some(format!("new {}()", args[0]))
                } else if m1 == msg::CALL_SIGNATURE_RETURN_TYPES_0_AND_1_ARE_INCOMPATIBLE.key {
                    Some(format!("{}(...)", args[0]))
                } else if m1 == msg::CONSTRUCT_SIGNATURE_RETURN_TYPES_0_AND_1_ARE_INCOMPATIBLE.key {
                    Some(format!("new {}(...)", args[0]))
                } else {
                    None
                };
                if let Some(arg) = arg {
                    self.relater_error_chain.pop();
                    self.relater_error_chain.pop();
                    self.relater_error_chain.push(RelaterChainEntry {
                        message: msg::THE_TYPES_RETURNED_BY_0_ARE_INCOMPATIBLE_BETWEEN_THESE_TYPES,
                        args: vec![arg],
                    });
                    return;
                }

                if (m1 == msg::TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE.key
                    || m1 == msg::THE_TYPES_OF_0_ARE_INCOMPATIBLE_BETWEEN_THESE_TYPES.key
                    || m1 == msg::THE_TYPES_RETURNED_BY_0_ARE_INCOMPATIBLE_BETWEEN_THESE_TYPES.key)
                    && let Some(tail_args) = self.chain_args(1).map(|a| a[0].clone())
                {
                    let dotted = Self::property_chain_name(&args[0], &tail_args);
                    self.relater_error_chain.pop();
                    self.relater_error_chain.pop();
                    self.relater_error_chain.push(RelaterChainEntry {
                        message: msg::THE_TYPES_OF_0_ARE_INCOMPATIBLE_BETWEEN_THESE_TYPES,
                        args: vec![dotted],
                    });
                    return;
                }
            }
        }
        self.relater_error_chain
            .push(RelaterChainEntry { message, args });
    }

    pub(crate) fn chain_property_arg_name(&self, prop: &Arc<crate::ast::Symbol>) -> String {
        let decl = prop
            .value_declaration
            .clone()
            .or_else(|| prop.declarations.first().cloned());
        if let Some(d) = decl
            && let Some(name) = d.name()
            && name.kind == SyntaxKind::StringLiteral
            && let Some(f) = self.get_source_file_of_node(&d)
        {
            let start = name.loc.pos();
            let end = name.loc.end();
            if start < end && end <= f.text.len() {
                return f.text[start..end].to_string();
            }
        }
        prop.name.clone()
    }

    pub(crate) fn push_relation_head_with_tp_note(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        head: crate::diagnostics::Message,
        head_args: Vec<String>,
    ) {
        use crate::diagnostics::messages_generated as msg;

        let target_flags_view = if target.flags.contains(TypeFlags::IndexedAccess)
            && !source.flags.contains(TypeFlags::IndexedAccess)
        {
            match &target.data {
                crate::checker::types::TypeData::IndexedAccess(d) => d
                    .object_type
                    .as_ref()
                    .map(|o| o.flags)
                    .unwrap_or(target.flags),
                _ => target.flags,
            }
        } else {
            target.flags
        };
        if target_flags_view.contains(TypeFlags::TypeParameter) {
            let constraint = self.get_base_constraint_of_type(target);
            let constraint_ok = constraint
                .as_ref()
                .is_some_and(|c| self.is_type_assignable_to(source, c));
            if constraint_ok {
                let c = constraint.unwrap();
                let s = self.type_to_string(source);
                let t = self.type_to_string(target);
                let c_str = self.type_to_string(&c);
                self.relater_report_error(
                    msg::X_0_IS_ASSIGNABLE_TO_THE_CONSTRAINT_OF_TYPE_1_BUT_1_COULD_BE_INSTANTIATED_WITH_A_DIFFERENT_SUBTYPE_OF_CONSTRAINT_2,
                    vec![s, t, c_str],
                );
            } else {
                self.relater_error_chain.clear();
                let t = self.type_to_string(target);
                let s = self.type_to_string(source);
                self.relater_report_error(
                    msg::X_0_COULD_BE_INSTANTIATED_WITH_AN_ARBITRARY_TYPE_WHICH_COULD_BE_UNRELATED_TO_1,
                    vec![t, s],
                );
            }
        }
        self.relater_report_error(head, head_args);
    }

    pub(crate) fn is_deeply_nested_type(
        &self,
        t: &Arc<Type>,
        stack: &[Arc<Type>],
        max_depth: usize,
    ) -> bool {
        if stack.len() < max_depth {
            return false;
        }
        if t.flags.contains(TypeFlags::Intersection) {
            if let Some(constituents) = t.types() {
                for c in constituents {
                    if self.is_deeply_nested_type(c, stack, max_depth) {
                        return true;
                    }
                }
            }
            return false;
        }
        let mut count = 0usize;
        let mut last_ptr: *const Type = std::ptr::null();
        for s in stack {
            let same = match (&t.symbol, &s.symbol) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => Arc::ptr_eq(t, s),
                _ => false,
            };
            if same {
                let p = Arc::as_ptr(s);
                if p != last_ptr {
                    count += 1;
                    if count >= max_depth {
                        return true;
                    }
                }
                last_ptr = p;
            }
        }
        false
    }
}
