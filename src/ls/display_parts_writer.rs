#![allow(dead_code)]

use crate::ast::Symbol;

#[derive(Debug, Clone)]
pub struct VsClassifiedTextRun {
    pub classification_type_name: String,
    pub text: String,
}

pub mod classification_type_name {
    pub const TEXT: &str = "text";
    pub const KEYWORD: &str = "keyword";
    pub const WHITE_SPACE: &str = "whitespace";
    pub const STRING: &str = "string";
    pub const OPERATOR: &str = "operator";
    pub const PARAMETER_NAME: &str = "parameter name";
    pub const PROPERTY_NAME: &str = "property name";
    pub const PUNCTUATION: &str = "punctuation";
    pub const LOCAL_NAME: &str = "local name";
    pub const FIELD_NAME: &str = "field name";
    pub const METHOD_NAME: &str = "method name";
    pub const CLASS_NAME: &str = "class name";
    pub const INTERFACE_NAME: &str = "interface name";
    pub const ENUM_NAME: &str = "enum name";
    pub const MODULE_NAME: &str = "module name";
    pub const TYPE_PARAMETER_NAME: &str = "type parameter name";
    pub const IDENTIFIER: &str = "identifier";
}

pub struct DisplayPartsWriter {
    builder: String,
    runs: Vec<VsClassifiedTextRun>,
    vs_capability: bool,
    last_written: String,
}

pub fn new_display_parts_writer(vs_capability: bool) -> DisplayPartsWriter {
    DisplayPartsWriter {
        builder: String::new(),
        runs: Vec::new(),
        vs_capability,
        last_written: String::new(),
    }
}

impl DisplayPartsWriter {
    pub fn add_run(&mut self, classification: &str, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.vs_capability {
            self.runs.push(VsClassifiedTextRun {
                classification_type_name: classification.to_string(),
                text: text.to_string(),
            });
        }
        self.last_written = text.to_string();
        self.builder.push_str(text);
    }

    pub fn write_classified(&mut self, text: &str, classification: &str) {
        self.add_run(classification, text);
    }

    pub fn write_from(&mut self, other: &DisplayPartsWriter) {
        self.builder.push_str(&other.builder);
        if self.vs_capability {
            self.runs.extend(other.runs.iter().cloned());
        }
        if !other.last_written.is_empty() {
            self.last_written = other.last_written.clone();
        }
    }

    pub fn get_runs(&self) -> &[VsClassifiedTextRun] {
        &self.runs
    }

    pub fn as_string(&self) -> &str {
        &self.builder
    }

    pub fn clear(&mut self) {
        self.last_written.clear();
        self.builder.clear();
        self.runs.clear();
    }

    pub fn has_trailing_whitespace(&self) -> bool {
        if self.builder.is_empty() {
            return false;
        }
        self.last_written
            .chars()
            .last()
            .map(crate::stringutil::is_white_space_like)
            .unwrap_or(false)
    }

    pub fn write(&mut self, s: &str) {
        self.add_run(classification_type_name::TEXT, s);
    }

    pub fn write_comment(&mut self, text: &str) {
        self.add_run(classification_type_name::TEXT, text);
    }

    pub fn write_keyword(&mut self, text: &str) {
        self.add_run(classification_type_name::KEYWORD, text);
    }

    pub fn write_line(&mut self) {
        self.add_run(classification_type_name::WHITE_SPACE, " ");
    }

    pub fn write_line_force(&mut self, _force: bool) {
        self.add_run(classification_type_name::WHITE_SPACE, " ");
    }

    pub fn write_literal(&mut self, s: &str) {
        self.add_run(classification_type_name::STRING, s);
    }

    pub fn write_operator(&mut self, text: &str) {
        self.add_run(classification_type_name::OPERATOR, text);
    }

    pub fn write_parameter(&mut self, text: &str) {
        self.add_run(classification_type_name::PARAMETER_NAME, text);
    }

    pub fn write_property(&mut self, text: &str) {
        self.add_run(classification_type_name::PROPERTY_NAME, text);
    }

    pub fn write_punctuation(&mut self, text: &str) {
        self.add_run(classification_type_name::PUNCTUATION, text);
    }

    pub fn write_space(&mut self, text: &str) {
        self.add_run(classification_type_name::WHITE_SPACE, text);
    }

    pub fn write_string_literal(&mut self, text: &str) {
        self.add_run(classification_type_name::STRING, text);
    }

    pub fn write_symbol(&mut self, text: &str, symbol: &Symbol) {
        let classification = classification_for_symbol(symbol);
        self.add_run(classification, text);
    }

    pub fn write_trailing_semicolon(&mut self, text: &str) {
        self.add_run(classification_type_name::PUNCTUATION, text);
    }
}

impl std::fmt::Display for DisplayPartsWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.builder)
    }
}

pub fn classification_for_symbol(symbol: &Symbol) -> &'static str {
    use crate::ast::SymbolFlags;
    let flags = symbol.flags;
    if flags.contains(SymbolFlags::VARIABLE) {
        if is_first_declaration_of_symbol_parameter(symbol) {
            return classification_type_name::PARAMETER_NAME;
        }
        return classification_type_name::LOCAL_NAME;
    }
    if flags.contains(SymbolFlags::Property) {
        return classification_type_name::PROPERTY_NAME;
    }
    if flags.contains(SymbolFlags::GetAccessor) {
        return classification_type_name::PROPERTY_NAME;
    }
    if flags.contains(SymbolFlags::SetAccessor) {
        return classification_type_name::PROPERTY_NAME;
    }
    if flags.contains(SymbolFlags::EnumMember) {
        return classification_type_name::FIELD_NAME;
    }
    if flags.contains(SymbolFlags::Function) {
        return classification_type_name::METHOD_NAME;
    }
    if flags.contains(SymbolFlags::Class) {
        return classification_type_name::CLASS_NAME;
    }
    if flags.contains(SymbolFlags::Interface) {
        return classification_type_name::INTERFACE_NAME;
    }
    if flags.contains(SymbolFlags::ENUM) {
        return classification_type_name::ENUM_NAME;
    }
    if flags.contains(SymbolFlags::NAMESPACE) {
        return classification_type_name::MODULE_NAME;
    }
    if flags.contains(SymbolFlags::Method) {
        return classification_type_name::METHOD_NAME;
    }
    if flags.contains(SymbolFlags::TypeParameter) {
        return classification_type_name::TYPE_PARAMETER_NAME;
    }
    if flags.contains(SymbolFlags::TypeAlias) {
        return classification_type_name::IDENTIFIER;
    }
    if flags.contains(SymbolFlags::Alias) {
        return classification_type_name::IDENTIFIER;
    }
    classification_type_name::TEXT
}

pub fn is_first_declaration_of_symbol_parameter(symbol: &Symbol) -> bool {
    use crate::ast::SyntaxKind;
    symbol
        .declarations
        .first()
        .map(|d| d.kind == SyntaxKind::Parameter)
        .unwrap_or(false)
}
