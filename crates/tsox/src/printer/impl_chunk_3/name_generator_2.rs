#![allow(unused_imports)]

use super::*;

impl NameGenerator {
    pub(crate) fn generate_name_for_module_or_enum(&mut self, node: &Arc<Node>) -> String {
        let name_node = node.name().expect("module/enum must have a name");
        let name = (self.get_text_of_node)(name_node);
        if let Some(ref check) = self.is_unique_local_name {
            if check(&name, node) {
                self.reserve_name(&name, false, false, false);
                return name;
            }
        }
        self.make_unique_name(&name, false, false, false, "", "")
    }

    pub(crate) fn generate_name_for_import_or_export_declaration(
        &mut self,
        node: &Arc<Node>,
    ) -> String {
        let base_name = get_external_module_name(node)
            .map(|s| make_identifier_from_module_name(&s))
            .unwrap_or_else(|| "module".to_string());
        self.make_unique_name(&base_name, false, false, false, "", "")
    }

    pub(crate) fn generate_name_for_export_default(&mut self) -> String {
        self.make_unique_name("default", false, false, false, "", "")
    }

    pub(crate) fn generate_name_for_method_or_accessor(
        &mut self,
        node: &Arc<Node>,
        private_name: bool,
        prefix: &str,
        suffix: &str,
    ) -> String {
        if let Some(name) = node.name() {
            if name.kind == SyntaxKind::Identifier {
                return self.generate_name_for_node_cached(
                    Some(name),
                    private_name,
                    GeneratedIdentifierFlags::NONE,
                    prefix,
                    suffix,
                );
            }
        }
        self.make_temp_variable_name(TEMP_FLAGS_AUTO, false, private_name, prefix, suffix)
    }

    pub(crate) fn make_name(&mut self, name: &GeneratedName) -> String {
        let auto_generate = &name.auto_generate;
        match auto_generate.flags.kind() {
            GeneratedIdentifierFlags::AUTO => self.make_temp_variable_name(
                TEMP_FLAGS_AUTO,
                auto_generate.flags.is_reserved_in_nested_scopes(),
                name.is_private,
                &auto_generate.prefix,
                &auto_generate.suffix,
            ),
            GeneratedIdentifierFlags::LOOP => self.make_temp_variable_name(
                TEMP_FLAGS_I,
                auto_generate.flags.is_reserved_in_nested_scopes(),
                false,
                &auto_generate.prefix,
                &auto_generate.suffix,
            ),
            GeneratedIdentifierFlags::UNIQUE => self.make_unique_name(
                name.text(),
                auto_generate.flags.is_optimistic(),
                auto_generate.flags.is_reserved_in_nested_scopes(),
                name.is_private,
                &auto_generate.prefix,
                &auto_generate.suffix,
            ),
            _ => name.text().to_string(),
        }
    }

    pub(crate) fn make_temp_variable_name(
        &mut self,
        flags: i32,
        reserved_in_nested_scopes: bool,
        private_name: bool,
        prefix: &str,
        suffix: &str,
    ) -> String {
        let simple = prefix.is_empty() && suffix.is_empty();
        let key = if simple {
            String::new()
        } else {
            let k = format_generated_name(private_name, prefix, "", suffix);
            if private_name {
                ensure_leading_hash(&k)
            } else {
                k
            }
        };

        let mut temp_flags = if simple {
            self.get_temp_flags(private_name)
        } else {
            self.get_temp_flags_for_formatted_name(private_name, &key)
        };

        if flags != 0 && temp_flags & flags == 0 {
            let full_name = format_generated_name(private_name, prefix, "_i", suffix);
            if self.is_unique_name(&full_name, private_name) {
                temp_flags |= flags;
                self.reserve_name(&full_name, private_name, reserved_in_nested_scopes, true);
                if simple {
                    self.set_temp_flags(private_name, temp_flags);
                } else {
                    self.set_temp_flags_for_formatted_name(private_name, key, temp_flags);
                }
                return full_name;
            }
        }

        loop {
            let count = temp_flags & TEMP_FLAGS_COUNT_MASK;
            temp_flags += 1;
            if count != 8 && count != 13 {
                let name = if count < 26 {
                    format!("_{}", (b'a' + count as u8) as char)
                } else {
                    format!("_{}", count - 26)
                };
                let full_name = format_generated_name(private_name, prefix, &name, suffix);
                if self.is_unique_name(&full_name, private_name) {
                    self.reserve_name(&full_name, private_name, reserved_in_nested_scopes, true);
                    if simple {
                        self.set_temp_flags(private_name, temp_flags);
                    } else {
                        self.set_temp_flags_for_formatted_name(private_name, key, temp_flags);
                    }
                    return full_name;
                }
            }
        }
    }

    pub(crate) fn make_unique_name(
        &mut self,
        base_name: &str,
        optimistic: bool,
        scoped: bool,
        private_name: bool,
        prefix: &str,
        suffix: &str,
    ) -> String {
        let base_name = remove_leading_hash(base_name);
        if optimistic {
            let full_name = format_generated_name(private_name, prefix, &base_name, suffix);
            if self.check_unique_name(&full_name, private_name) {
                self.reserve_name(&full_name, private_name, scoped, false);
                return full_name;
            }
        }

        let mut base_name = base_name.to_string();
        if !base_name.is_empty() && !base_name.ends_with('_') {
            base_name.push('_');
        }

        let mut i = 1;
        loop {
            let full_name =
                format_generated_name(private_name, prefix, &format!("{base_name}{i}"), suffix);
            if self.check_unique_name(&full_name, private_name) {
                self.reserve_name(&full_name, private_name, scoped, false);
                return full_name;
            }
            i += 1;
        }
    }
}
