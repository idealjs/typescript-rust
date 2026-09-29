use std::cmp::Ordering;
use std::sync::Arc;

use crate::ast::diagnostic::Diagnostic;
use crate::ast::diagnostic::DiagnosticsCollection;
use crate::ast::node_data_generated::NodeData;
use crate::ast::node_node::Node;
use crate::ast::syntax_kind_generated::SyntaxKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepopulateDiagnosticKind {
    ModeMismatch = 1,
    ModuleNotFound = 2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepopulateDiagnosticInfo {
    pub kind: RepopulateDiagnosticKind,
    pub module_reference: String,
    pub mode: tsox_core::core::compiler_options::ResolutionMode,
    pub package_name: String,
}

impl Diagnostic {
    pub fn set_repopulate_info(&mut self, info: Option<RepopulateDiagnosticInfo>) {
        let _ = info;
    }

    pub fn set_skipped_on_no_emit(&mut self) {
        self.skipped_on_no_emit = true;
    }

    pub fn skipped_on_no_emit(&self) -> bool {
        self.skipped_on_no_emit
    }

    pub fn source(&self) -> &str {
        ""
    }

    pub fn message_text(&self) -> &str {
        self.message_key
    }

    pub fn display_string(&self) -> String {
        if self.message.is_none() && !self.message_text().is_empty() {
            return self.message_text().to_string();
        }
        let args = self.display_message_args();
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        self.message
            .as_ref()
            .expect("message or message_text required")
            .localize(&tsox_core::locale::Locale::default_locale(), &arg_refs)
    }

    pub fn display_message_args(&self) -> Vec<String> {
        let Some(_file) = &self.file else {
            return self.message_args.clone();
        };
        if !self.source().is_empty() {
            return self.message_args.clone();
        }
        self.message_args.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticLocationKey {
    pub path: String,
    pub loc: tsox_core::core::text::TextRange,
    pub code: i32,
}

pub fn get_diagnostic_location_key(diagnostic: &Diagnostic) -> DiagnosticLocationKey {
    let path = diagnostic
        .file
        .as_ref()
        .map(|f| f.file_name.clone())
        .unwrap_or_default();
    DiagnosticLocationKey {
        path,
        loc: diagnostic.loc,
        code: diagnostic.code,
    }
}

pub fn get_diagnostic_path(d: &Diagnostic) -> String {
    d.file
        .as_ref()
        .map(|f| f.file_name.clone())
        .unwrap_or_default()
}

pub fn get_diagnostic_message_identity(diagnostic: &Diagnostic) -> String {
    if !diagnostic.message_text().is_empty() {
        return diagnostic.message_text().to_string();
    }
    if let Some(message) = &diagnostic.message {
        if diagnostic.code == -1 {
            return message.to_string();
        }
    }
    diagnostic.message_key.to_string()
}

pub fn equal_message_chain(c1: &Diagnostic, c2: &Diagnostic) -> bool {
    c1.code == c2.code
        && c1.message_args == c2.message_args
        && c1.message_chain.len() == c2.message_chain.len()
        && c1
            .message_chain
            .iter()
            .zip(c2.message_chain.iter())
            .all(|(a, b)| equal_message_chain(a, b))
}

pub fn compare_message_chain_size(c1: &[Diagnostic], c2: &[Diagnostic]) -> Ordering {
    let mut c = c2.len().cmp(&c1.len());
    if c != Ordering::Equal {
        return c;
    }
    for i in 0..c1.len() {
        c = compare_message_chain_size(&c1[i].message_chain, &c2[i].message_chain);
        if c != Ordering::Equal {
            return c;
        }
    }
    Ordering::Equal
}

pub fn compare_message_chain_content(c1: &[Diagnostic], c2: &[Diagnostic]) -> Ordering {
    for i in 0..c1.len() {
        let mut c = compare_str_slices(&c1[i].message_args, &c2[i].message_args);
        if c != Ordering::Equal {
            return c;
        }
        c = compare_message_chain_content(&c1[i].message_chain, &c2[i].message_chain);
        if c != Ordering::Equal {
            return c;
        }
    }
    Ordering::Equal
}

pub fn compare_related_info(r1: &[Diagnostic], r2: &[Diagnostic]) -> Ordering {
    let mut c = r2.len().cmp(&r1.len());
    if c != Ordering::Equal {
        return c;
    }
    for i in 0..r1.len() {
        c = compare_diagnostics(&r1[i], &r2[i]);
        if c != Ordering::Equal {
            return c;
        }
    }
    Ordering::Equal
}

fn compare_str_slices(a: &[String], b: &[String]) -> Ordering {
    for (x, y) in a.iter().zip(b.iter()) {
        let c = x.as_str().cmp(y.as_str());
        if c != Ordering::Equal {
            return c;
        }
    }
    a.len().cmp(&b.len())
}

pub fn compare_diagnostics(d1: &Diagnostic, d2: &Diagnostic) -> Ordering {
    let c = get_diagnostic_path(d1).cmp(&get_diagnostic_path(d2));
    if c != Ordering::Equal {
        return c;
    }
    let c = d1.loc.pos().cmp(&d2.loc.pos());
    if c != Ordering::Equal {
        return c;
    }
    let c = d1.loc.end().cmp(&d2.loc.end());
    if c != Ordering::Equal {
        return c;
    }
    let c = d1.code.cmp(&d2.code);
    if c != Ordering::Equal {
        return c;
    }
    let c = (d1.category as i32).cmp(&(d2.category as i32));
    if c != Ordering::Equal {
        return c;
    }
    let c = d1.source().cmp(d2.source());
    if c != Ordering::Equal {
        return c;
    }
    let c = get_diagnostic_message_identity(d1).cmp(&get_diagnostic_message_identity(d2));
    if c != Ordering::Equal {
        return c;
    }
    let c = compare_str_slices(&d1.message_args, &d2.message_args);
    if c != Ordering::Equal {
        return c;
    }
    let c = compare_message_chain_size(&d1.message_chain, &d2.message_chain);
    if c != Ordering::Equal {
        return c;
    }
    let c = compare_message_chain_content(&d1.message_chain, &d2.message_chain);
    if c != Ordering::Equal {
        return c;
    }
    compare_related_info(&d1.related_information, &d2.related_information)
}

impl DiagnosticsCollection {
    pub fn get_global_diagnostics_locked(&mut self) -> Vec<Diagnostic> {
        let inner = self.inner.get_mut().unwrap();
        if !inner.non_file_diagnostics_sorted {
            inner
                .non_file_diagnostics
                .sort_by(|a, b| compare_diagnostics(a, b));
            inner.non_file_diagnostics_sorted = true;
        }
        inner.non_file_diagnostics.clone()
    }

    pub fn get_diagnostics_for_file_locked(&mut self, file: &crate::ast::node_source_file::SourceFile) -> Vec<Diagnostic> {
        let inner = self.inner.get_mut().unwrap();
        if !inner.file_diagnostics_sorted.contains(&file.file_name) {
            if let Some(bucket) = inner.file_diagnostics.get_mut(&file.file_name) {
                bucket.sort_by(|a, b| compare_diagnostics(a, b));
            }
            inner.file_diagnostics_sorted.insert(file.file_name.clone());
        }
        inner
            .file_diagnostics
            .get(&file.file_name)
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FlowFlags(pub u32);

impl FlowFlags {
    pub const UNREACHABLE: FlowFlags = FlowFlags(1 << 0);
    pub const START: FlowFlags = FlowFlags(1 << 1);
    pub const BRANCH_LABEL: FlowFlags = FlowFlags(1 << 2);
    pub const LOOP_LABEL: FlowFlags = FlowFlags(1 << 3);
    pub const ASSIGNMENT: FlowFlags = FlowFlags(1 << 4);
    pub const TRUE_CONDITION: FlowFlags = FlowFlags(1 << 5);
    pub const FALSE_CONDITION: FlowFlags = FlowFlags(1 << 6);
    pub const SWITCH_CLAUSE: FlowFlags = FlowFlags(1 << 7);
    pub const ARRAY_MUTATION: FlowFlags = FlowFlags(1 << 8);
    pub const CALL: FlowFlags = FlowFlags(1 << 9);
    pub const REDUCE_LABEL: FlowFlags = FlowFlags(1 << 10);
    pub const REFERENCED: FlowFlags = FlowFlags(1 << 11);
    pub const SHARED: FlowFlags = FlowFlags(1 << 12);
    pub const LABEL: FlowFlags = FlowFlags((1 << 2) | (1 << 3));
    pub const CONDITION: FlowFlags = FlowFlags((1 << 5) | (1 << 6));

    pub fn contains(self, other: FlowFlags) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug)]
pub struct FlowSwitchClauseData {
    pub switch_statement: Arc<Node>,
    pub clause_start: i32,
    pub clause_end: i32,
}

impl FlowSwitchClauseData {
    pub fn new_node(
        switch_statement: Arc<Node>,
        clause_start: usize,
        clause_end: usize,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::Unknown,
            crate::ast::node_data_generated::NodeData::FlowSwitchClauseData(FlowSwitchClauseData {
                switch_statement,
                clause_start: clause_start as i32,
                clause_end: clause_end as i32,
            }),
        ))
    }

    pub fn is_empty(&self) -> bool {
        self.clause_start == self.clause_end
    }
}

#[derive(Debug)]
pub struct FlowReduceLabelData {
    pub target: Arc<crate::ast::symbol_flow::FlowLabel>,
    pub antecedents: Option<Vec<Arc<crate::ast::symbol_flow::FlowNode>>>,
}

impl FlowReduceLabelData {
    pub fn new_node(
        target: Arc<crate::ast::symbol_flow::FlowLabel>,
        antecedents: Option<Vec<Arc<crate::ast::symbol_flow::FlowNode>>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::Unknown,
            crate::ast::node_data_generated::NodeData::FlowReduceLabelData(FlowReduceLabelData {
                target,
                antecedents,
            }),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FunctionFlags(pub u32);

impl FunctionFlags {
    pub const NORMAL: FunctionFlags = FunctionFlags(0);
    pub const GENERATOR: FunctionFlags = FunctionFlags(1 << 0);
    pub const ASYNC: FunctionFlags = FunctionFlags(1 << 1);
    pub const INVALID: FunctionFlags = FunctionFlags(1 << 2);
    pub const ASYNC_GENERATOR: FunctionFlags = FunctionFlags((1 << 1) | (1 << 0));

    pub fn contains(self, other: FunctionFlags) -> bool {
        self.0 & other.0 == other.0
    }
}

pub fn get_function_flags(node: Option<&Arc<Node>>) -> FunctionFlags {
    let Some(node) = node else {
        return FunctionFlags::INVALID;
    };
    let (asterisk_token, body) = match &node.data {
        NodeData::FunctionDeclaration(d) => (d.asterisk_token.clone(), d.body.clone()),
        NodeData::FunctionExpression(d) => (d.asterisk_token.clone(), Some(d.body.clone())),
        NodeData::MethodDeclaration(d) => (d.asterisk_token.clone(), d.body.clone()),
        NodeData::ArrowFunction(d) => (None, Some(d.body.clone())),
        NodeData::ConstructorDeclaration(d) => (None, d.body.clone()),
        NodeData::GetAccessorDeclaration(d) => (None, d.body.clone()),
        NodeData::SetAccessorDeclaration(d) => (None, d.body.clone()),
        _ => return FunctionFlags::INVALID,
    };
    let mut flags = FunctionFlags::NORMAL;
    match node.kind {
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::MethodDeclaration => {
            if asterisk_token.is_some() {
                flags.0 |= FunctionFlags::GENERATOR.0;
            }
            if node.has_syntactic_modifier(crate::ast::node_flags::ModifierFlags::Async) {
                flags.0 |= FunctionFlags::ASYNC.0;
            }
        }
        SyntaxKind::ArrowFunction => {
            if node.has_syntactic_modifier(crate::ast::node_flags::ModifierFlags::Async) {
                flags.0 |= FunctionFlags::ASYNC.0;
            }
        }
        _ => {}
    }
    if body.is_none() {
        flags.0 |= FunctionFlags::INVALID.0;
    }
    flags
}
