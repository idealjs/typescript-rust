use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_function_type() {
    let content = r#"const /*constDefinition*/c: () => void;
/*constReference*/c();
function test(/*cbDefinition*/cb: () => void) {
    /*cbReference*/cb();
}
class C {
    /*propDefinition*/prop: () => void;
    m() {
        this./*propReference*/prop();
    }
}"#;
    let _s = Session::new_for_test("goToDefinitionFunctionType", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "constReference", "cbReference", "propReference")
}
