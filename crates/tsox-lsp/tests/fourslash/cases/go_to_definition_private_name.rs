use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_private_name() {
    let content = r#"class A {
    [|/*pnMethodDecl*/#method|]() { }
    [|/*pnFieldDecl*/#foo|] = 3;
    get [|/*pnPropGetDecl*/#prop|]() { return ""; }
    set [|/*pnPropSetDecl*/#prop|](value: string) {  }
    constructor() {
        this.[|/*pnFieldUse*/#foo|]
        this.[|/*pnMethodUse*/#method|]
        this.[|/*pnPropUse*/#prop|]
    }
}"#;
    let mut s = Session::new_for_test("goToDefinitionPrivateName", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "pnFieldUse", "pnMethodUse", "pnPropUse")
}
