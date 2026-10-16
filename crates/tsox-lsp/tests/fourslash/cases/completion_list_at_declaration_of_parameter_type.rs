use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionListAtDeclarationOfParameterType", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["Bar"], &[]);
}
