use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_items_module_variables() {
    let content = r#"// @Filename: navigationItemsModuleVariables_0.ts
 /*file1*/
namespace Module1 {
    export var x = 0;
}
// @Filename: navigationItemsModuleVariables_1.ts
 /*file2*/
namespace Module1.SubModule {
    export var y = 0;
}
// @Filename: navigationItemsModuleVariables_2.ts
 /*file3*/
namespace Module1 {
    export var z = 0;
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "file1");
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_marker(&mut s, "file2");
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
