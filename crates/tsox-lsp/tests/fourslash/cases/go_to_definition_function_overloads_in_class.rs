use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_function_overloads_in_class() {
    let content = r#"class clsInOverload {
    static fnOverload();
    static [|/*staticFunctionOverload*/fnOverload|](foo: string);
    static /*staticFunctionOverloadDefinition*/fnOverload(foo: any) { }
    public [|/*functionOverload*/fnOverload|](): any;
    public fnOverload(foo: string);
    public /*functionOverloadDefinition*/fnOverload(foo: any) { return "foo" }

    constructor() { }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "staticFunctionOverload", "functionOverload")
}
