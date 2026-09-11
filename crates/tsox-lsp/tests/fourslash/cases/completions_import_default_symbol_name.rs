use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_default_symbol_name() {
    let content = r#"// @module: commonjs
// @esModuleInterop: false
// @allowSyntheticDefaultImports: false
// @Filename: /node_modules/@types/range-parser/index.d.ts
declare function RangeParser(): string;
declare namespace RangeParser {
    interface Options {
        combine?: boolean;
    }
}
export = RangeParser;
// @Filename: /b.ts
R/*0*/"#;
    let mut s = Session::new_for_test("completionsImport_default_symbolName", content);
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new("0"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
