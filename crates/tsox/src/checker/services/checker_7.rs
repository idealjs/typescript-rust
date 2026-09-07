#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn is_lib_symbol_for_hover_verbosity(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if let Some(sf) = self.get_source_file_of_node(decl) {
                if self.program.is_source_file_default_library(&sf.file_name) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_lib_type_for_hover_verbosity(&self, t: &Arc<Type>) -> bool {
        let symbol = if t.object_flags.contains(ObjectFlags::Reference) {
            t.target().and_then(|target| target.symbol.clone())
        } else {
            t.symbol.clone()
        };
        if let Some(ref sym) = symbol {
            if self.is_lib_symbol_for_hover_verbosity(sym) {
                return true;
            }
        }
        is_tuple_type(t)
    }

    pub fn resolve_external_module_symbol(
        &self,
        module_symbol: &Arc<Symbol>,
        _dont_resolve_alias: bool,
    ) -> Arc<Symbol> {
        if let Some(export_equals) = module_symbol.exports.get("export=") {
            return Arc::clone(export_equals);
        }
        Arc::clone(module_symbol)
    }

    pub fn get_members_of_symbol(&self, symbol: &Arc<Symbol>) -> SymbolTable {
        symbol.members.clone()
    }

    pub fn remove_optional_type_marker(&self, t: &Arc<Type>) -> Arc<Type> {
        Arc::clone(t)
    }

    pub fn get_index_type_of_type(
        &self,
        t: &Arc<Type>,
        _index_kind: IndexKind,
    ) -> Option<Arc<Type>> {
        if let Some(structured) = t.as_structured() {
            for info in &structured.index_infos {
                if let Some(ref key_type) = info.key_type {
                    let matches = match _index_kind {
                        IndexKind::String => key_type.flags.contains(TypeFlags::String),
                        IndexKind::Number => key_type.flags.contains(TypeFlags::Number),
                    };
                    if matches {
                        return info.value_type.clone();
                    }
                }
            }
        }
        None
    }

    pub fn get_apparent_type(&self, t: &Arc<Type>) -> Arc<Type> {
        Arc::clone(t)
    }

    pub fn get_reduced_apparent_type(&self, t: &Arc<Type>) -> Arc<Type> {
        self.get_apparent_type(t)
    }

    pub fn resolve_structured_type_members(&self, t: &Arc<Type>) -> Arc<Type> {
        Arc::clone(t)
    }

    pub fn is_named_member(&self, _symbol: &Arc<Symbol>, _name: &str) -> bool {
        !is_reserved_member_name(_name)
    }

    pub fn get_named_members(
        &self,
        props_by_name: &std::collections::HashMap<String, Arc<Symbol>>,
    ) -> Vec<Arc<Symbol>> {
        props_by_name
            .values()
            .filter(|s| !is_reserved_member_name(&s.name))
            .cloned()
            .collect()
    }

    pub fn is_property_accessible(
        &self,
        _node: &Arc<Node>,
        _is_super: bool,
        _is_write: bool,
        _t: &Arc<Type>,
        _property: &Arc<Symbol>,
    ) -> bool {
        true
    }

    pub(crate) fn get_widened_type_of_expression(&mut self, expr: &Arc<Node>) -> Arc<Type> {
        let t = self.get_type_of_node(expr);
        self.get_widened_type(&t)
    }

    pub fn get_type_of_property_of_type(&mut self, t: &Arc<Type>, name: &str) -> Option<Arc<Type>> {
        if let Some(prop) = self.get_property_of_type(t, name) {
            return Some(self.get_type_of_symbol(&prop));
        }
        None
    }

    pub fn get_literal_type_from_property_name(
        &mut self,
        property_name: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        match property_name.kind {
            SyntaxKind::StringLiteral => Some(self.get_string_literal_type(property_name.text())),
            SyntaxKind::NumericLiteral => None,
            SyntaxKind::PrivateIdentifier => None,
            SyntaxKind::ComputedPropertyName => None,
            _ => None,
        }
    }

    pub fn is_this_type_parameter(&self, t: &Arc<Type>) -> bool {
        if !t.flags.contains(TypeFlags::TypeParameter) {
            return false;
        }
        if let TypeData::TypeParameter(tp) = &t.data {
            tp.is_this_type
        } else {
            false
        }
    }

    pub fn get_contextual_type_for_element_expression(
        &mut self,
        _contextual_type: &Arc<Type>,
        _element_index: usize,
        _length: Option<usize>,
        _first_spread_index: i32,
        _last_spread_index: i32,
    ) -> Option<Arc<Type>> {
        None
    }

    pub(crate) fn global_callable_function_type(&self) -> Option<Arc<Type>> {
        None
    }

    pub(crate) fn global_newable_function_type(&self) -> Option<Arc<Type>> {
        None
    }

    pub(crate) fn get_jsx_type_symbol(&self, _name: &str, _location: &Arc<Node>) -> Option<Arc<Type>> {
        None
    }

    pub fn get_source_file_of_node(&self, node: &Arc<Node>) -> Option<Arc<SourceFile>> {
        let mut current = Arc::clone(node);
        loop {
            if current.kind == SyntaxKind::SourceFile {
                let node_id = current.id();
                for file in &self.files {
                    if file.node.id() == node_id {
                        return Some(Arc::clone(file));
                    }
                }
                return None;
            }
            current = current.parent.clone()?;
        }
    }
}

pub(crate) fn get_possible_symbol_reference_nodes(
    source_file: &Arc<SourceFile>,
    symbol_name: &str,
    container: Option<&Arc<Node>>,
) -> Vec<Arc<Node>> {
    let positions = get_possible_symbol_reference_positions(source_file, symbol_name, container);
    let mut result = Vec::new();
    for pos in positions {
        if let Some(node) = find_identifier_at_pos(source_file, pos) {
            result.push(node);
        }
    }
    result
}

pub(crate) fn get_possible_symbol_reference_positions(
    source_file: &Arc<SourceFile>,
    symbol_name: &str,
    container: Option<&Arc<Node>>,
) -> Vec<usize> {
    let mut positions = Vec::new();

    if symbol_name.is_empty() {
        return positions;
    }

    let text = source_file.text.as_str();
    let symbol_name_len = symbol_name.len();

    let search_start = container.and_then(|c| Some(c.pos())).unwrap_or(0);
    let end_pos = container.and_then(|c| Some(c.end())).unwrap_or(text.len());

    let mut search_from = search_start;
    while search_from < end_pos {
        let remainder = &text[search_from..end_pos];
        let relative_pos = match remainder.find(symbol_name) {
            Some(p) => p,
            None => break,
        };
        let position = search_from + relative_pos;
        let end_position = position + symbol_name_len;

        let prev_ok = position == 0 || !is_identifier_part_byte(text.as_bytes()[position - 1]);
        let next_ok =
            end_position >= text.len() || !is_identifier_part_byte(text.as_bytes()[end_position]);

        if prev_ok && next_ok {
            positions.push(position);
        }

        search_from = position + symbol_name_len + 1;
        if search_from > text.len() {
            break;
        }
    }

    positions
}

pub(crate) fn is_identifier_part_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'$' || b == b'_'
}

pub(crate) fn find_identifier_at_pos(source_file: &Arc<SourceFile>, pos: usize) -> Option<Arc<Node>> {
    let file_node = &source_file.node;
    find_node_at_pos(file_node, pos)
}

pub(crate) fn find_node_at_pos(node: &Arc<Node>, pos: usize) -> Option<Arc<Node>> {
    if node.pos() <= pos && pos < node.end() {
        if node.kind == SyntaxKind::Identifier && node.pos() == pos {
            return Some(Arc::clone(node));
        }

        let mut found = None;
        crate::ast::for_each_child(node, |child| {
            if found.is_none() {
                if let Some(f) = find_node_at_pos(child, pos) {
                    found = Some(Arc::clone(&f));
                    return true;
                }
            }
            false
        });
        return found;
    }
    None
}

pub(crate) fn is_array_literal_or_object_literal_destructuring_pattern(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ArrayLiteralExpression | SyntaxKind::ObjectLiteralExpression
    ) && node
        .parent
        .as_ref()
        .map(|p| {
            p.kind == SyntaxKind::BinaryExpression
                || p.kind == SyntaxKind::ForOfStatement
                || p.kind == SyntaxKind::VariableDeclaration
        })
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKind {
    String,
    Number,
}
