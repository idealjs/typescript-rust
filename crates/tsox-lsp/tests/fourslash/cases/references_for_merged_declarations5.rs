use tsox_lsp::fourslash::Session;


#[test]
fn references_for_merged_declarations5() {
    let content = r#"interface /*1*/Foo { }
module /*2*/Foo { export interface Bar { } }
function /*3*/Foo() { }

export = /*4*/Foo;"#;
    let _s = Session::new_for_test("referencesForMergedDeclarations5", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
