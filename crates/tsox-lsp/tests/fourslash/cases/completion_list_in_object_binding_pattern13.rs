use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_object_binding_pattern13() {
    let content = r#"interface I {
    x: number;
    y: string;
    z: boolean;
}

interface J {
    x: string;
    y: string;
}

let { /**/ }: I | J = { x: 10 };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
