use crate::lsp::lsproto::{Diagnostic, IntegerOrString};

impl IntegerOrString {
    pub fn as_string(&self) -> String { ::tsox_core::fntrace::enter("as_string"); 
        if let Some(s) = &self.string {
            return s.clone();
        }
        if let Some(i) = &self.integer {
            return i.to_string();
        }
        "-1".to_string()
    }
}

impl Diagnostic {
    pub fn code_string(&self) -> String { ::tsox_core::fntrace::enter("code_string"); 
        match &self.code {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
            None => "-1".to_string(),
        }
    }

    pub fn code_as_string(&self) -> String { ::tsox_core::fntrace::enter("code_as_string"); 
        format!("Code({})", self.code_string())
    }

    pub fn as_string(&self) -> String { ::tsox_core::fntrace::enter("as_string"); 
        format!(
            "{} ({}:{}-{}:{}): {}",
            self.code_string(),
            self.range.start.line,
            self.range.start.character,
            self.range.end.line,
            self.range.end.character,
            self.message
        )
    }
}
