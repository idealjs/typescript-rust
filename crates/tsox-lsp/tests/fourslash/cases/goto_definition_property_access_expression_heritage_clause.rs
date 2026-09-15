use tsox_lsp::fourslash::Session;


#[test]
fn goto_definition_property_access_expression_heritage_clause() {
    let content = r#"class B {}
function foo() {
    return {/*refB*/B: B};
}
class C extends (foo()).[|/*B*/B|] {}
class C1 extends foo().[|/*B1*/B|] {}"#;
    let _s = Session::new_for_test("gotoDefinitionPropertyAccessExpressionHeritageClause", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "B", "B1")
}
