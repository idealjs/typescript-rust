#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_source_file(&mut self, file: &Arc<SourceFile>) {
        self.type_instantiation_count = 0;

        if !self.globals_populated {
            self.populate_globals();
            self.globals_populated = true;
        }

        let file_node = Arc::clone(&file.node);
        let file_id = file_node.id();
        let source_file_symbol = self.program.symbol_map().symbol_of(&file_node).cloned();

        self.set_parent_pointers(&file_node);

        let file_arc = Arc::clone(file);
        self.current_file = Some(Arc::clone(&file_arc));
        self.current_file_id = file_id;
        self.current_file_symbol = source_file_symbol;

        self.push_scope(&file_node);

        let statements: Vec<Arc<Node>> = match &file_node.data {
            crate::ast::NodeData::SourceFile(data) => data.statements.iter().cloned().collect(),
            _ => Vec::new(),
        };

        self.check_function_overloads_recursive(&statements);
        for stmt in &statements {
            self.check_statement(stmt);
        }

        self.check_export_assignment_conflicts(&statements);

        self.check_unused_identifiers_in_file(&file_node);

        self.pop_scope();
        self.current_file = None;
        self.current_file_id = 0;
        self.current_file_symbol = None;
    }

    pub fn get_semantic_diagnostics(&self) -> Vec<crate::ast::Diagnostic> {
        self.diagnostics.get_all()
    }
}
