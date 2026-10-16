use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_type_of1() {
    let content = r#"namespace m1c {
    export interface I { foo(): void; }
}
var x: typeof m1c./*1*/;"#;
    let mut s = Session::new_for_test("completionInTypeOf1", content);
    fourslash::verify_completions_empty_at(&mut s, Some("1"));
}
