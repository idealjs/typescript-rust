use std::sync::Mutex;

use tsox_core::core::text::TextRange;
use tsox_core::diagnostics::Category;
use tsox_core::diagnostics::Message;

use super::SourceFile;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub file: Option<std::sync::Arc<SourceFile>>,
    pub loc: TextRange,
    pub code: i32,
    pub category: Category,
    pub message: Option<Message>,
    pub message_key: tsox_core::diagnostics::Key,
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

#[derive(Debug, Default)]
pub struct DiagnosticsCollection {
    inner: Mutex<DiagnosticsCollectionInner>,
}

#[derive(Debug, Default)]
pub struct DiagnosticsCollectionInner {
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
        if Self::is_duplicate(&inner, &diagnostic) {
            return;
        }
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

    /// Go DiagnosticsCollection.Add：同 file+loc+code 且全等（消息/实参/链/related）
    /// 的诊断只保留第一条
    fn is_duplicate(inner: &DiagnosticsCollectionInner, diagnostic: &Diagnostic) -> bool {
        let candidates: &[Diagnostic] = match diagnostic.file.as_ref() {
            Some(file) => inner
                .file_diagnostics
                .get(&file.file_name)
                .map(|bucket| bucket.as_slice())
                .unwrap_or(&[]),
            None => &inner.non_file_diagnostics,
        };
        candidates
            .iter()
            .any(|d| d.loc == diagnostic.loc && d.code == diagnostic.code && equal_diagnostics(d, diagnostic))
    }

    pub fn add_or_append_related(&self, mut diagnostic: Diagnostic) {
        let mut inner = self.inner.lock().unwrap();
        let file_name = diagnostic.file.as_ref().map(|f| f.file_name.clone());
        let matches = |d: &Diagnostic| {
            d.code == diagnostic.code
                && d.loc == diagnostic.loc
                && d.file.as_ref().map(|f| f.file_name.as_str()) == file_name.as_deref()
        };
        let mut appended = false;
        {
            let existing = match file_name.as_ref() {
                Some(name) => inner
                    .file_diagnostics
                    .get_mut(name)
                    .and_then(|bucket| bucket.iter_mut().find(|d| matches(d))),
                None => inner.non_file_diagnostics.iter_mut().find(|d| matches(d)),
            };
            if let Some(existing) = existing {
                let related = std::mem::take(&mut diagnostic.related_information);
                for rel in related {
                    let already = existing.related_information.iter().any(|r| {
                        r.loc == rel.loc
                            && r.file.as_ref().map(|f| f.file_name.as_str())
                                == rel.file.as_ref().map(|f| f.file_name.as_str())
                    });
                    if !already {
                        existing.related_information.push(rel);
                    }
                }
                appended = true;
            }
        }
        if !appended {
            drop(inner);
            self.add(diagnostic);
        }
    }

    pub fn count(&self) -> usize {
        self.inner.lock().unwrap().count
    }

    pub fn take_inner(&self) -> DiagnosticsCollectionInner {
        std::mem::take(&mut *self.inner.lock().unwrap())
    }

    pub fn set_inner(&self, inner: DiagnosticsCollectionInner) {
        *self.inner.lock().unwrap() = inner;
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    pub fn get_all(&self) -> Vec<Diagnostic> {
        let inner = self.inner.lock().unwrap();
        let mut result = Vec::with_capacity(inner.count);
        result.extend(inner.non_file_diagnostics.iter().cloned());
        for diags in inner.file_diagnostics.values() {
            result.extend(diags.iter().cloned());
        }
        result
    }

    pub fn get_for_file(&self, file_name: &str) -> Vec<Diagnostic> {
        let inner = self.inner.lock().unwrap();
        inner
            .file_diagnostics
            .get(file_name)
            .cloned()
            .unwrap_or_default()
    }
}

impl Diagnostic {
    pub fn with_text(self, text: impl Into<String>) -> Diagnostic {
        Diagnostic {
            file: self.file,
            loc: self.loc,
            code: self.code,
            category: self.category,
            message: None,
            message_key: self.message_key,
            message_args: vec![text.into()],
            message_chain: self.message_chain,
            related_information: self.related_information,
            reports_unnecessary: self.reports_unnecessary,
            reports_deprecated: self.reports_deprecated,
            skipped_on_no_emit: self.skipped_on_no_emit,
        }
    }
}

/// Go EqualDiagnostics：消息身份（key+实参）+ 位置 + 类别 + 链 + related 全等
fn equal_diagnostics(a: &Diagnostic, b: &Diagnostic) -> bool {
    a.message_key == b.message_key
        && a.message_args == b.message_args
        && a.category == b.category
        && a.message_chain.len() == b.message_chain.len()
        && a
            .message_chain
            .iter()
            .zip(b.message_chain.iter())
            .all(|(x, y)| {
                x.loc == y.loc
                    && x.code == y.code
                    && x.message_key == y.message_key
                    && x.message_args == y.message_args
            })
        && a.related_information.len() == b.related_information.len()
        && a
            .related_information
            .iter()
            .zip(b.related_information.iter())
            .all(|(x, y)| {
                x.loc == y.loc
                    && x.code == y.code
                    && x.message_key == y.message_key
                    && x.message_args == y.message_args
            })
}
