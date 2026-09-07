use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_shorthand_object_literal_with_interface() {
    let content = r#"interface Something {
    [|foo|]: string;
}

function makeSomething([|foo|]: string): Something {
    return { [|f/*1*/oo|] };
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
