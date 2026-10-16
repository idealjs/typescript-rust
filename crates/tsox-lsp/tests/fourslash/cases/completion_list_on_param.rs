use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_on_param() {
    let content = r#"namespace Bar {
    export class Blah { }
}

class Point {
    public Foo(x: Bar./**/Blah, y: Bar.Blah) { }
}"#;
    let mut s = Session::new_for_test("completionListOnParam", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["Blah"]);
}
