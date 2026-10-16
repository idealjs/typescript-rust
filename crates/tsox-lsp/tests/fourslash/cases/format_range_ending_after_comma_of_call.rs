use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_range_ending_after_comma_of_call() {
    let content = r#"someCall(
    /*start*/"firstParameter",/*end*/
    "something else"
);"#;
    let mut s = Session::new_for_test("formatRangeEndingAfterCommaOfCall", content);
    fourslash::format_selection(&mut s, "start", "end");
}
