use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_merged_pattern_ambient_module() {
    let content = r#"// @Filename: /tsconfig.json
{ "compilerOptions": { "module": "preserve", "moduleResolution": "bundler" } }

// @Filename: /first.d.ts
declare module "*.asset" with { type: "css" } {
    export const styles: string;
}

// @Filename: /second.d.ts
declare module "*.asset" with { type: "css" } {
    export const styleTokens: string;
}

// @Filename: /index.ts
sty/**/"#;
    let mut s = Session::new_for_test("autoImportMergedPatternAmbientModule", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
