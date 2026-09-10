use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatSelection"]
#[test]
fn format_selection_single_property() {
    let content = r#"console.log({
}, {
/*1*/    a: 1,
/*2*/    b: 2
})"#;
    let mut s = Session::new_for_test("formatSelectionSingleProperty", content);
    fourslash::unsupported("FormatSelection"); // f.FormatSelection(t, "1", "2")
    fourslash::verify_current_file_content(&mut s, r#"console.log({
}, {
    a: 1,
    b: 2
})"#);
}
