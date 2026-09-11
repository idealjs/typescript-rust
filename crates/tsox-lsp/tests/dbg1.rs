use tsox_frontend::ast::NodeData;

#[test]
fn probe_class_in_ctor() {
    let text = "class K {\n  #value: number;\n  foo() {\n     this.#;\n  }\n}";
    let file = tsox_frontend::parser::Parser::parse_source_file_text("t.ts", text.to_string());
    dump(&file.node, 0);
    fn dump(n: &std::sync::Arc<tsox_frontend::ast::Node>, depth: usize) {
        if depth < 12 {
            let name = n.name().map(|x| x.text().to_string()).unwrap_or_default();
            println!("{}{:?} {}", "  ".repeat(depth), n.kind, name);
        }
        tsox_frontend::ast::node_data_generated::for_each_child(n, |c| {
            dump(c, depth + 1);
            false
        });
    }
}
