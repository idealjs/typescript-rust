use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn format_space_between_function_and_array_index() {
    let content = r#"// @lib: es5

function test() {
    return [];
}

test() [0]
"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"
function test() {
    return [];
}

test()[0]
"#,
    );
}
