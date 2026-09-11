use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_css_module() {
    let content = r#"
// @Filename: /tsconfig.json
{ "compilerOptions": { "module": "nodenext", "moduleResolution": "nodenext" } }

// @Filename: /package.json
{ "type": "module" }

// @Filename: /augmentations.ts
export {};
declare module "./styles.css" {
    export const myClass: string;
}

// @Filename: /index.ts
myClass/**/
"#;
    let mut s = Session::new_for_test("autoImportCssModule", content);
    // TODO: // Verify auto-import completions don't panic when importing from .css module augmentation
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
