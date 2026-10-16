#![allow(unused_imports)]

use tsox_frontend::ast;

pub fn leading_indentation(text: &str) -> String { ::tsox_core::fntrace::enter("leading_indentation"); 
    let end = text
        .bytes()
        .take_while(|&b| b == b' ' || b == b'\t')
        .count();
    text[..end].to_string()
}

pub fn line_has_only_jsdoc_asterisk(line: &str) -> bool { ::tsox_core::fntrace::enter("line_has_only_jsdoc_asterisk"); 
    let line = line.trim_start_matches([' ', '\t']);
    match line.strip_prefix('*') {
        Some(rest) => rest.bytes().all(|b| b == b' ' || b == b'\t'),
        None => false,
    }
}

pub fn needs_parenthesized_expression_for_assertion(node: &ast::Node) -> bool { ::tsox_core::fntrace::enter("needs_parenthesized_expression_for_assertion"); 
    !ast::is_entity_name_expression(node)
        && !ast::is_call_expression(node)
        && !ast::is_object_literal_expression(node)
        && !ast::is_array_literal_expression(node)
}
