//! Diagnostic types, ported from `internal/ast/diagnostic.go`.

use std::sync::Mutex;

use crate::core::text::TextRange;
use crate::diagnostics::{self, Category, Message};

use super::SourceFile;

/// A diagnostic message attached to a source location.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub file: Option<std::sync::Arc<SourceFile>>,
    pub loc: TextRange,
    pub code: i32,
    pub category: Category,
    pub message: Option<Message>,
    pub message_key: diagnostics::Key,
    pub message_args: Vec<String>,
    pub message_chain: Vec<Diagnostic>,
    pub related_information: Vec<Diagnostic>,
    pub reports_unnecessary: bool,
    pub reports_deprecated: bool,
    pub skipped_on_no_emit: bool,
}

impl Diagnostic {
    pub fn new(
        file: Option<std::sync::Arc<SourceFile>>,
        loc: TextRange,
        message: Message,
        args: Vec<String>,
    ) -> Self {
        Self {
            file,
            loc,
            code: message.code,
            category: message.category,
            message: Some(message),
            message_key: message.key,
            message_args: args,
            message_chain: Vec::new(),
            related_information: Vec::new(),
            reports_unnecessary: message.reports_unnecessary,
            reports_deprecated: message.reports_deprecated,
            skipped_on_no_emit: false,
        }
    }

    pub fn is_error(&self) -> bool {
        self.category == Category::Error
    }
}

/// A thread-safe collection of diagnostics, organized by file.
#[derive(Debug, Default)]
pub struct DiagnosticsCollection {
    inner: Mutex<DiagnosticsCollectionInner>,
}

#[derive(Debug, Default)]
struct DiagnosticsCollectionInner {
    count: usize,
    file_diagnostics: std::collections::HashMap<String, Vec<Diagnostic>>,
    file_diagnostics_sorted: std::collections::HashSet<String>,
    non_file_diagnostics: Vec<Diagnostic>,
    non_file_diagnostics_sorted: bool,
}

impl DiagnosticsCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&self, diagnostic: Diagnostic) {
        let mut inner = self.inner.lock().unwrap();
        inner.count += 1;
        if let Some(file) = &diagnostic.file {
            let file_name = file.file_name.clone();
            inner
                .file_diagnostics
                .entry(file_name)
                .or_default()
                .push(diagnostic);
            inner.file_diagnostics_sorted.clear();
        } else {
            inner.non_file_diagnostics.push(diagnostic);
            inner.non_file_diagnostics_sorted = false;
        }
    }

    pub fn count(&self) -> usize {
        self.inner.lock().unwrap().count
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Get all diagnostics as a flat list.
    pub fn get_all(&self) -> Vec<Diagnostic> {
        let inner = self.inner.lock().unwrap();
        let mut result = Vec::with_capacity(inner.count);
        result.extend(inner.non_file_diagnostics.iter().cloned());
        for diags in inner.file_diagnostics.values() {
            result.extend(diags.iter().cloned());
        }
        result
    }

    /// Get diagnostics for a specific file.
    pub fn get_for_file(&self, file_name: &str) -> Vec<Diagnostic> {
        let inner = self.inner.lock().unwrap();
        inner
            .file_diagnostics
            .get(file_name)
            .cloned()
            .unwrap_or_default()
    }
}
