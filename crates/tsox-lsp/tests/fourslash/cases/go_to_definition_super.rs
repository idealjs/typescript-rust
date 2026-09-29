use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_super() {
    let content = r#"class A {
    /*ctr*/constructor() {}
    x() {}
}
class /*B*/B extends A {}
class C extends B {
    constructor() {
        [|/*super*/super|]();
    }
    method() {
        [|/*superExpression*/super|].x();
    }
}
class D {
    constructor() {
        /*superBroken*/super();
    }
}"#;
    let _s = Session::new_for_test("goToDefinition_super", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "super", "superExpression", "superBroken")
}
