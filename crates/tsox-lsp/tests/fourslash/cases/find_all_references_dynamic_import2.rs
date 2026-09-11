use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_references_dynamic_import2() {
    let content = r#"// @Filename: foo.ts
[|export function /*1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}bar|]() { return "bar"; }|]
var x = import("./foo");
x.then(foo => {
    foo./*2*/[|bar|]();
})"#;
    let mut s = Session::new_for_test("findAllReferencesDynamicImport2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "bar")
}
