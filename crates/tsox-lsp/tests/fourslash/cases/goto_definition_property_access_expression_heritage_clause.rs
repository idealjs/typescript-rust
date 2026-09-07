use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn goto_definition_property_access_expression_heritage_clause() {
    let content = r#"class B {}
function foo() {
    return {/*refB*/B: B};
}
class C extends (foo()).[|/*B*/B|] {}
class C1 extends foo().[|/*B1*/B|] {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "B", "B1")
}
