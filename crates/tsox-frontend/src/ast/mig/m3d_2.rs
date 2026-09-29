#![allow(unused_imports)]
use std::sync::Arc;
use crate::ast::node::{ModifierList, Node, NodeList};
use crate::ast::node_flags::NodeFlags;
use super::m3d::{NodeFactory, NodeVisitor, NodeVisitorHooks};
use crate::ast::diagnostic::{Diagnostic, DiagnosticsCollection};
use crate::ast::node::SourceFile;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics::{new_ad_hoc_message, Category, Key, Message};
use tsox_core::locale::Locale;
use crate::ast::deep_clone_node::deep_clone_node;
use super::m3g_3::set_parent_in_children;

fn force_mut<T: ?Sized>(arc: &mut Arc<T>) -> &mut T {
    let ptr: *mut T = match Arc::get_mut(arc) {
        Some(v) => v as *mut T,
        None => Arc::as_ptr(arc) as *mut T,
    };
    unsafe { &mut *ptr }
}

fn clone_node_list(list: &NodeList) -> NodeList {
    NodeList { loc: list.loc, nodes: list.nodes.clone() }
}

fn clone_modifier_list(list: &ModifierList) -> ModifierList {
    ModifierList { list: clone_node_list(&list.list), modifier_flags: list.modifier_flags }
}

fn relocate_list_tail(list: &mut NodeList, has_trailing_comma: bool) {
    if has_trailing_comma {
        if let Some(last) = list.nodes.last_mut() {
            force_mut(last).loc = TextRange { pos: -2, end: -2 };
        }
    }
    list.loc = TextRange { pos: -1, end: -1 };
}

pub fn get_deep_clone_visitor(f: &NodeFactory, synthetic_location: bool) -> NodeVisitor {
    let mut visitor: Option<NodeVisitor> = None;
    let v = NodeVisitor::new(
        move |node: &Arc<Node>| -> Arc<Node> {
            let visitor = visitor.as_ref().unwrap();
            let visited = visitor.visit_each_child_node(node);
            if !Arc::ptr_eq(&visited, node) {
                let mut visited = match Arc::try_unwrap(visited) {
                    Ok(owned) => Arc::new(owned),
                    Err(shared) => deep_clone_node(&shared),
                };
                if synthetic_location {
                    force_mut(&mut visited).loc = TextRange { pos: -1, end: -1 };
                }
                return visited;
            }
            let mut c = deep_clone_node(node);
            if synthetic_location {
                force_mut(&mut c).loc = TextRange { pos: -1, end: -1 };
            }
            c
        },
        f,
        NodeVisitorHooks {
            visit_nodes: Some(Box::new(move |nodes: Option<&Arc<NodeList>>, v: &NodeVisitor| -> Option<Arc<NodeList>> {
                let nodes = nodes?;
                let visited = v.visit_nodes(nodes);
                let new_list = if !Arc::ptr_eq(&visited, nodes) {
                    visited
                } else {
                    Arc::new(clone_node_list(nodes))
                };
                if synthetic_location {
                    let mut new_list = match Arc::try_unwrap(new_list) {
                        Ok(owned) => owned,
                        Err(shared) => clone_node_list(&shared),
                    };
                    relocate_list_tail(&mut new_list, nodes.has_trailing_comma());
                    return Some(Arc::new(new_list));
                }
                Some(new_list)
            })),
            visit_modifiers: Some(
                Box::new(move |nodes: Option<&Arc<ModifierList>>, v: &NodeVisitor| -> Option<Arc<ModifierList>> {
                    let nodes = nodes?;
                    let visited = v.visit_modifiers(nodes);
                    let new_list = if !Arc::ptr_eq(&visited, nodes) {
                        visited
                    } else {
                        Arc::new(clone_modifier_list(nodes))
                    };
                    if synthetic_location {
                        let mut new_list = match Arc::try_unwrap(new_list) {
                            Ok(owned) => owned,
                            Err(shared) => clone_modifier_list(&shared),
                        };
                        relocate_list_tail(&mut new_list.list, nodes.has_trailing_comma());
                        return Some(Arc::new(new_list));
                    }
                    Some(new_list)
                },
            )),
        },
    );
    visitor = Some(v);
    visitor.unwrap()
}

impl NodeFactory {
    pub fn deep_clone_reparse(&self, node: Option<Arc<Node>>) -> Option<Arc<Node>> {
        let mut node = node?;
        let visited = get_deep_clone_visitor(self, false).visit_node(&node);
        set_parent_in_children(&visited);
        node = match Arc::try_unwrap(visited) {
            Ok(mut owned) => {
                owned.flags |= NodeFlags::Reparsed;
                Arc::new(owned)
            }
            Err(shared) => shared,
        };
        Some(node)
    }

    pub fn deep_clone_reparse_modifiers(&self, modifiers: &Arc<ModifierList>) -> Arc<ModifierList> {
        get_deep_clone_visitor(self, false).visit_modifiers(modifiers)
    }
}

#[derive(Clone)]
pub struct RepopulateDiagnosticInfo {
    pub kind: RepopulateDiagnosticKind,
    pub module_reference: String,
    pub mode: tsox_core::core::compiler_options_kinds::ResolutionMode,
    pub package_name: String,
}

#[derive(Clone)]
pub enum RepopulateDiagnosticKind {
    ModeMismatch = 1,
    ModuleNotFound = 2,
}

impl Diagnostic {
    pub fn file(&self) -> Option<Arc<SourceFile>> {
        self.file.clone()
    }

    pub fn pos(&self) -> i32 {
        self.loc.pos
    }

    pub fn end(&self) -> i32 {
        self.loc.end
    }

    pub fn len(&self) -> i32 {
        self.loc.end - self.loc.pos
    }

    pub fn loc(&self) -> TextRange {
        self.loc
    }

    pub fn code(&self) -> i32 {
        self.code
    }

    pub fn category(&self) -> Category {
        self.category
    }

    pub fn message_key(&self) -> Key {
        self.message_key
    }

    pub fn message_args(&self) -> &[String] {
        &self.message_args
    }

    pub fn message_chain(&self) -> &[Diagnostic] {
        &self.message_chain
    }

    pub fn related_information(&self) -> &[Diagnostic] {
        &self.related_information
    }

    pub fn reports_unnecessary(&self) -> bool {
        self.reports_unnecessary
    }

    pub fn reports_deprecated(&self) -> bool {
        self.reports_deprecated
    }

    pub fn repopulate_info(&self) -> Option<&RepopulateDiagnosticInfo> {
        None
    }

    pub fn set_file(&mut self, file: Option<Arc<SourceFile>>) {
        self.file = file;
    }

    pub fn set_location(&mut self, loc: TextRange) {
        self.loc = loc;
    }

    pub fn set_category(&mut self, category: Category) {
        self.category = category;
    }

    pub fn set_external_data(&mut self, source: String, message_text: String) -> &mut Diagnostic {
        let _ = (source, message_text);
        self
    }

    pub fn set_message_chain(&mut self, message_chain: Vec<Diagnostic>) -> &mut Diagnostic {
        self.message_chain = message_chain;
        self
    }

    pub fn add_message_chain(&mut self, message_chain: Diagnostic) -> &mut Diagnostic {
        self.message_chain.push(message_chain);
        self
    }

    pub fn set_related_info(&mut self, related_information: Vec<Diagnostic>) -> &mut Diagnostic {
        self.related_information = related_information;
        self
    }

    pub fn add_related_info(&mut self, related_information: Diagnostic) -> &mut Diagnostic {
        self.related_information.push(related_information);
        self
    }

    pub fn localize(&self, locale: Locale) -> String {
        if self.message.is_none() && !self.message_text().is_empty() {
            return self.message_text().to_string();
        }
        let Some(message) = self.message.as_ref() else {
            return String::new();
        };
        let args = self.display_message_args();
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        message.localize(&locale, &arg_refs)
    }
}

pub fn new_diagnostic_from_serialized(
    file: Option<Arc<SourceFile>>,
    loc: TextRange,
    code: i32,
    category: Category,
    message_key: Key,
    message_args: Vec<String>,
    message_chain: Vec<Diagnostic>,
    related_information: Vec<Diagnostic>,
    reports_unnecessary: bool,
    reports_deprecated: bool,
    skipped_on_no_emit: bool,
) -> Diagnostic {
    Diagnostic {
        file,
        loc,
        code,
        category,
        message: None,
        message_key,
        message_args,
        message_chain,
        related_information,
        reports_unnecessary,
        reports_deprecated,
        skipped_on_no_emit,
    }
}

pub fn new_diagnostic_from_text(
    file: Option<Arc<SourceFile>>,
    loc: TextRange,
    code: i32,
    category: Category,
    text: String,
    message_chain: Vec<Diagnostic>,
    related_information: Vec<Diagnostic>,
    reports_unnecessary: bool,
    reports_deprecated: bool,
) -> Diagnostic {
    Diagnostic {
        file,
        loc,
        code,
        category,
        message: Some(new_ad_hoc_message(Box::leak(text.into_boxed_str()))),
        message_key: Key::default(),
        message_args: Vec::new(),
        message_chain,
        related_information,
        reports_unnecessary,
        reports_deprecated,
        skipped_on_no_emit: false,
    }
}

pub fn new_diagnostic_chain(
    chain: Option<&Diagnostic>,
    message: Message,
    args: Vec<String>,
) -> Diagnostic {
    if let Some(chain) = chain {
        let mut d = Diagnostic::new(chain.file.clone(), chain.loc, message, args);
        d.add_message_chain(chain_clone_shallow(chain));
        d.related_information = clone_related(chain.related_information());
        return d;
    }
    Diagnostic::new(None, TextRange::default(), message, args)
}

fn chain_clone_shallow(chain: &Diagnostic) -> Diagnostic {
    clone_diagnostic_shallow(chain)
}

fn clone_related(related: &[Diagnostic]) -> Vec<Diagnostic> {
    related.iter().map(clone_diagnostic_shallow).collect()
}

pub fn clone_diagnostic_shallow(d: &Diagnostic) -> Diagnostic {
    d.clone()
}

pub fn new_compiler_diagnostic(message: Message, args: Vec<String>) -> Diagnostic {
    Diagnostic::new(None, TextRange::undefined(), message, args)
}

pub fn new_external_diagnostic(
    file: Option<Arc<SourceFile>>,
    loc: TextRange,
    _source: String,
    category: Category,
    code: i32,
    message_text: String,
) -> Diagnostic {
    let mut d = Diagnostic::new(
        None,
        loc,
        new_ad_hoc_message(Box::leak(message_text.into_boxed_str())),
        Vec::new(),
    );
    d.file = file;
    d.code = code;
    d.category = category;
    d
}

impl DiagnosticsCollection {
    pub fn lookup(&self, diagnostic: &Diagnostic) -> Option<Diagnostic> {
        let inner = self.inner.lock().unwrap();
        let diagnostics: &[Diagnostic] = match diagnostic.file.as_ref() {
            Some(file) => inner
                .file_diagnostics
                .get(&file.file_name)
                .map(|v| v.as_slice())
                .unwrap_or(&[]),
            None => {
                if !inner.non_file_diagnostics_sorted {
                    drop(inner);
                    let _ = self.get_global_diagnostics();
                    return self.lookup_post_sort(diagnostic);
                }
                &inner.non_file_diagnostics
            }
        };
        binary_search_diagnostic(diagnostics, diagnostic).cloned()
    }

    fn lookup_post_sort(&self, diagnostic: &Diagnostic) -> Option<Diagnostic> {
        let inner = self.inner.lock().unwrap();
        binary_search_diagnostic(&inner.non_file_diagnostics, diagnostic).cloned()
    }

    pub fn get_global_diagnostics(&self) -> Vec<Diagnostic> {
        let mut inner = self.inner.lock().unwrap();
        if !inner.non_file_diagnostics_sorted {
            inner.non_file_diagnostics.sort_by(compare_diagnostics);
            inner.non_file_diagnostics_sorted = true;
        }
        inner.non_file_diagnostics.clone()
    }
}

fn binary_search_diagnostic<'a>(
    diagnostics: &'a [Diagnostic],
    target: &Diagnostic,
) -> Option<&'a Diagnostic> {
    let mut low = 0usize;
    let mut high = diagnostics.len();
    while low < high {
        let mid = (low + high) / 2;
        match compare_diagnostics(&diagnostics[mid], target) {
            std::cmp::Ordering::Equal => return Some(&diagnostics[mid]),
            std::cmp::Ordering::Less => low = mid + 1,
            std::cmp::Ordering::Greater => high = mid,
        }
    }
    None
}

fn get_diagnostic_path(d: &Diagnostic) -> &str {
    if let Some(file) = &d.file {
        return &file.file_name;
    }
    ""
}

pub fn equal_diagnostics(d1: &Diagnostic, d2: &Diagnostic) -> bool {
    if std::ptr::eq(d1, d2) {
        return true;
    }
    equal_diagnostics_no_related_info(d1, d2)
        && d1.related_information.len() == d2.related_information.len()
        && d1
            .related_information
            .iter()
            .zip(d2.related_information.iter())
            .all(|(r1, r2)| equal_diagnostics(r1, r2))
}

pub fn equal_diagnostics_no_related_info(d1: &Diagnostic, d2: &Diagnostic) -> bool {
    if std::ptr::eq(d1, d2) {
        return true;
    }
    get_diagnostic_path(d1) == get_diagnostic_path(d2)
        && d1.loc == d2.loc
        && d1.code == d2.code
        && d1.category == d2.category
        && d1.source() == d2.source()
        && get_diagnostic_message_identity(d1) == get_diagnostic_message_identity(d2)
        && d1.message_args == d2.message_args
        && d1.message_chain.len() == d2.message_chain.len()
        && d1
            .message_chain
            .iter()
            .zip(d2.message_chain.iter())
            .all(|(c1, c2)| equal_message_chain(c1, c2))
}

fn get_diagnostic_message_identity(diagnostic: &Diagnostic) -> String {
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

fn equal_message_chain(c1: &Diagnostic, c2: &Diagnostic) -> bool {
    if std::ptr::eq(c1, c2) {
        return true;
    }
    c1.code == c2.code
        && c1.message_args == c2.message_args
        && c1.message_chain.len() == c2.message_chain.len()
        && c1
            .message_chain
            .iter()
            .zip(c2.message_chain.iter())
            .all(|(a, b)| equal_message_chain(a, b))
}

fn compare_message_chain_size(c1: &[Diagnostic], c2: &[Diagnostic]) -> std::cmp::Ordering {
    let c = c2.len().cmp(&c1.len());
    if c != std::cmp::Ordering::Equal {
        return c;
    }
    for i in 0..c1.len() {
        let c = compare_message_chain_size(&c1[i].message_chain, &c2[i].message_chain);
        if c != std::cmp::Ordering::Equal {
            return c;
        }
    }
    std::cmp::Ordering::Equal
}

fn compare_message_chain_content(c1: &[Diagnostic], c2: &[Diagnostic]) -> std::cmp::Ordering {
    for i in 0..c1.len() {
        let c = c1[i].message_args.cmp(&c2[i].message_args);
        if c != std::cmp::Ordering::Equal {
            return c;
        }
        if !c1[i].message_chain.is_empty() {
            let c = compare_message_chain_content(&c1[i].message_chain, &c2[i].message_chain);
            if c != std::cmp::Ordering::Equal {
                return c;
            }
        }
    }
    std::cmp::Ordering::Equal
}

fn compare_related_info(r1: &[Diagnostic], r2: &[Diagnostic]) -> std::cmp::Ordering {
    let c = r2.len().cmp(&r1.len());
    if c != std::cmp::Ordering::Equal {
        return c;
    }
    for i in 0..r1.len() {
        let c = compare_diagnostics(&r1[i], &r2[i]);
        if c != std::cmp::Ordering::Equal {
            return c;
        }
    }
    std::cmp::Ordering::Equal
}

pub fn compare_diagnostics(d1: &Diagnostic, d2: &Diagnostic) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    if std::ptr::eq(d1, d2) {
        return Ordering::Equal;
    }
    let c = get_diagnostic_path(d1).cmp(get_diagnostic_path(d2));
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
    let c = d1.message_args.cmp(&d2.message_args);
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
