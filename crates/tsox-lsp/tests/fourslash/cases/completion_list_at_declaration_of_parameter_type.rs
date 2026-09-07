use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_at_declaration_of_parameter_type() {
    let content = r#"namespace Bar {
    export class Bleah {
    }
    export class Foo extends Bleah {
    }
}

function Blah(x: /**/Bar.Bleah) {
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
