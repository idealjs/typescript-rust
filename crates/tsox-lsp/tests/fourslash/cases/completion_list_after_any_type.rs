use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_any_type() {
    let content = r#"declare class myString {
    charAt(pos: number): string;
}

function bar(a: myString) {
    var x: any = a./**/
}"#;
    let mut s = Session::new_for_test("completionListAfterAnyType", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["charAt"]);
}
