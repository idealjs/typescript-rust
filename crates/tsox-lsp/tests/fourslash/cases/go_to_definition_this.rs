use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_this() {
    let content = r#"function f(/*fnDecl*/this: number) {
    return [|/*fnUse*/this|];
}
class /*cls*/C {
    constructor() { return [|/*clsUse*/this|]; }
    get self(/*getterDecl*/this: number) { return [|/*getterUse*/this|]; }
}"#;
    let mut s = Session::new_for_test("goToDefinitionThis", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "fnUse", "clsUse", "getterUse")
}
