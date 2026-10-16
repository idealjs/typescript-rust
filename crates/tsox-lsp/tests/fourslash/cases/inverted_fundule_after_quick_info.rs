use tsox_lsp::fourslash::{self, Session};


#[test]
fn inverted_fundule_after_quick_info() {
    let content = r#"namespace M {
    namespace A {
        var o;
    }
    function A(/**/x: number): void { }
}"#;
    let mut s = Session::new_for_test("invertedFunduleAfterQuickInfo", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoExists(t)
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
