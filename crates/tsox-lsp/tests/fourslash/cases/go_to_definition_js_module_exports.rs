use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_js_module_exports() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
x./*def*/test = () => { }
x.[|/*ref*/test|]();
x./*defFn*/test3 = function () { }
x.[|/*refFn*/test3|]();"#;
    let mut s = Session::new_for_test("goToDefinitionJsModuleExports", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "ref", "refFn")
}
