use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_typeof_expressions() {
    let content = r#"const x = "str";
function test(arg: typeof x./*1*/) {}
function test1(arg: typeof (x./*2*/)) {}"#;
    let mut s = Session::new_for_test("completionTypeofExpressions", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["length"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("2"), &["length"], &[]);
}
