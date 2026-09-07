#![allow(unused_imports)]

use crate::checker::checker_modules::*;

impl Checker {
    pub(crate) fn module_can_have_synthetic_default(
        &mut self,
        module_symbol: &Arc<Symbol>,
    ) -> bool {
        if self.module_has_syntactic_default(module_symbol) {
            return false;
        }
        if module_symbol.exports.get("__esModule").is_some() {
            return false;
        }
        let is_ambient_or_declaration = module_symbol.declarations.iter().any(|d| match &d.data {
            tsox_frontend::ast::NodeData::ModuleDeclaration(_) => true,
            tsox_frontend::ast::NodeData::SourceFile(_) => self
                .get_source_file_of_node(d)
                .is_some_and(|f| f.is_declaration_file),
            _ => false,
        });
        if is_ambient_or_declaration {
            return true;
        }
        module_symbol.exports.get("export=").is_some()
    }

    pub(crate) fn declaring_dir_of(&self, node: &Arc<Node>) -> Option<String> {
        self.get_source_file_of_node(node)
            .or_else(|| self.current_file.clone())
            .map(|f| match f.file_name.rfind('/') {
                Some(i) => f.file_name[..i].to_string(),
                None => String::new(),
            })
    }
}
