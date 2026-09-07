use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatSelection"]
#[test]
fn format_range_ending_after_comma_of_call() {
    let content = r#"someCall(
    /*start*/"firstParameter",/*end*/
    "something else"
);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatSelection"); // f.FormatSelection(t, "start", "end")
}
