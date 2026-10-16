use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_on_super() {
    let content = r#"class TAB<T>{
    foo<T>(x: T) {
    }
    bar(a: number, b: number) {
    }
}

class TAD<T> extends TAB<T> {
    constructor() {
        super();
    }
    bar(f: number) {
        super./**/
    }
}"#;
    let mut s = Session::new_for_test("completionListOnSuper", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["bar", "foo"]);
}
