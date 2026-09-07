use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_namespace_06() {
    let content = r#"namespace [|F/*declaration*/oo|] {
    declare function hello(): void;
}


let x: typeof Foo = [|{ hello() {} }|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "declaration")
}
