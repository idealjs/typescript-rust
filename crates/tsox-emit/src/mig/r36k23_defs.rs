pub trait R36K23EmitContextExt {
    fn text_source(&self, node: &Node) -> Option<Arc<Node>>;
    fn most_original(&self, node: &Arc<Node>) -> Arc<Node>;
    fn has_auto_generate_info(&self, node: &Node) -> bool;
}

impl R36K23EmitContextExt for tsox_frontend::format::mig::m4o_2::EmitContext {
    fn text_source(&self, _node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("text_source"); 
        None
    }

    fn most_original(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("most_original"); 
        Arc::clone(node)
    }

    fn has_auto_generate_info(&self, _node: &Node) -> bool { ::tsox_core::fntrace::enter("has_auto_generate_info"); 
        false
    }
}

pub trait R36K23SourceFileExt {
    fn text(&self) -> &str;
    fn ecma_line_map(&self) -> Vec<i32>;
}

impl R36K23SourceFileExt for tsox_frontend::ast::node_source_file::SourceFile {
    fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        &self.text
    }

    fn ecma_line_map(&self) -> Vec<i32> { ::tsox_core::fntrace::enter("ecma_line_map"); 
        tsox_frontend::format::mig::m4t_3::get_ecma_line_starts(self)
    }
}

impl Printer {
    pub fn write_parameter(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_parameter"); 
        self.writer.write_parameter(text);
    }

    pub fn write_operator(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_operator"); 
        self.writer.write_operator(text);
    }

    pub fn write_property(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_property"); 
        self.writer.write_property(text);
    }

    pub fn write_literal(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_literal"); 
        self.writer.write_literal(text);
    }

    pub fn r36k23_source_file_is_current(&self, node: &Node) -> bool { ::tsox_core::fntrace::enter("r36k23_source_file_is_current"); 
        let Some(current) = self.current_source_file.as_ref() else {
            return false;
        };
        match r36k23_get_source_file_of_node(node) {
            Some(file) => {
                let original = self.emit_context.most_original(&current.node);
                Arc::ptr_eq(&file, &original)
            }
            None => false,
        }
    }
}

pub fn r36k23_get_source_file_of_node(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("r36k23_get_source_file_of_node"); 
    let mut current = node.parent();
    while let Some(n) = current {
        if n.kind == SyntaxKind::SourceFile {
            return Some(n);
        }
        current = n.parent();
    }
    None
}

pub fn r36k23_skip_synthesized_parentheses(node: &Node) -> &Node { ::tsox_core::fntrace::enter("r36k23_skip_synthesized_parentheses"); 
    let mut current = node;
    while current.kind == SyntaxKind::ParenthesizedExpression && node_is_synthesized(current) {
        match current.expression() {
            Some(expr) => current = expr,
            None => break,
        }
    }
    current
}
