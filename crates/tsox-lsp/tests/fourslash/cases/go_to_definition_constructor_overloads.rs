use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_constructor_overloads() {
    let content = r#"class ConstructorOverload {
    [|/*constructorOverload1*/constructor|]();
    /*constructorOverload2*/constructor(foo: string);
    /*constructorDefinition*/constructor(foo: any)  { }
}

var constructorOverload = new [|/*constructorOverloadReference1*/ConstructorOverload|]();
var constructorOverload = new [|/*constructorOverloadReference2*/ConstructorOverload|]("foo");

class Extended extends ConstructorOverload {
    readonly name = "extended";
}
var extended1 = new [|/*extendedRef1*/Extended|]();
var extended2 = new [|/*extendedRef2*/Extended|]("foo");"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "constructorOverloadReference1", "constructorOverloadReferen
}
