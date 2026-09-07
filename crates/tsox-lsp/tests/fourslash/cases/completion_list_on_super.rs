use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
