use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_type_of2() {
    let content = r#"namespace m1c {
    export class C { foo(): void; }
}
var x: typeof m1c./*1*/;"#;
    let mut s = Session::new_for_test("completionInTypeOf2", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["C"]);
}
