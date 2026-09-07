use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_after_class_extends() {
    let content = r#"namespace Bar {
    export class Bleah {
    }
    export class Foo extends /**/Bleah {
    }
}

function Blah(x: Bar.Bleah) {
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
