use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn auto_import_same_name_default_exported() {
    let content = r#"// @lib: es5
// @module: commonjs
// @Filename: /node_modules/antd/index.d.ts
declare function Table(): void;
export default Table;
// @Filename: /node_modules/rc-table/index.d.ts
declare function Table(): void;
export default Table;
// @Filename: /index.ts
Table/**/"#;
    let mut s = Session::new_for_test("autoImportSameNameDefaultExported", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
