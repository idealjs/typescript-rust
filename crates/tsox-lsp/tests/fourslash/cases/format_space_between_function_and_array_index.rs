use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_space_between_function_and_array_index() {
    let content = r#"// @lib: es5

function test() {
    return [];
}

test() [0]
"#;
    let mut s = Session::new_for_test("formatSpaceBetweenFunctionAndArrayIndex", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"
function test() {
    return [];
}

test()[0]
"#);
}
