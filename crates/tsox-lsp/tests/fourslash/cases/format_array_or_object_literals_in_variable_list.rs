use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_array_or_object_literals_in_variable_list() {
    let content = r#"var v30 = [1, 2], v31, v32, v33 = [0], v34 = {'a': true}, v35;/**/"#;
    let mut s = Session::new_for_test("formatArrayOrObjectLiteralsInVariableList", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"var v30 = [1, 2], v31, v32, v33 = [0], v34 = { 'a': true }, v35;"#);
}
