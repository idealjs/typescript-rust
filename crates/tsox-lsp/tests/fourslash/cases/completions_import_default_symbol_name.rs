use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("0"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
