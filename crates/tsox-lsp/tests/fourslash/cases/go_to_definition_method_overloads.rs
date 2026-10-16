use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_method_overloads() {
    let content = r#"class MethodOverload {
    static [|/*staticMethodOverload1*/method|]();
    static /*staticMethodOverload2*/method(foo: string);
    static /*staticMethodDefinition*/method(foo?: any) { }
    public [|/*instanceMethodOverload1*/method|](): any;
    public /*instanceMethodOverload2*/method(foo: string);
    public /*instanceMethodDefinition*/method(foo?: any) { return "foo" }
}
// static method
MethodOverload.[|/*staticMethodReference1*/method|]();
MethodOverload.[|/*staticMethodReference2*/method|]("123");
// instance method
var methodOverload = new MethodOverload();
methodOverload.[|/*instanceMethodReference1*/method|]();
methodOverload.[|/*instanceMethodReference2*/method|]("456");"#;
    let _s = Session::new_for_test("goToDefinitionMethodOverloads", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "staticMethodReference1", "staticMethodReference2", "instanc
}
