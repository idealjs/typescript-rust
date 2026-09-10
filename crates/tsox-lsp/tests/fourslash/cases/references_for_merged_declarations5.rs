use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_merged_declarations5() {
    let content = r#"interface /*1*/Foo { }
module /*2*/Foo { export interface Bar { } }
function /*3*/Foo() { }

export = /*4*/Foo;"#;
    let mut s = Session::new_for_test("referencesForMergedDeclarations5", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
