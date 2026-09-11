use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_typeof_this() {
    let content = r#"function f(/*fnDecl*/this: number) {
    type X = typeof [|/*fnUse*/this|];
}
class /*cls*/C {
    constructor() { type X = typeof [|/*clsUse*/this|]; }
    get self(/*getterDecl*/this: number) { type X = typeof [|/*getterUse*/this|]; }
}"#;
    let mut s = Session::new_for_test("goToDefinitionTypeofThis", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "fnUse", "clsUse", "getterUse")
}
