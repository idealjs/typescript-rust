use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn auto_import_file_exclude_patterns2() {
    let content = r#"// @lib: es5
// @Filename: /lib/components/button/Button.ts
export function Button() {}
// @Filename: /lib/components/button/index.ts
export * from "./Button";
// @Filename: /lib/components/index.ts
export * from "./button";
// @Filename: /lib/main.ts
export { Button } from "./components";
// @Filename: /lib/index.ts
export * from "./main";
// @Filename: /i-hate-index-files.ts
Button/**/"#;
    let mut s = Session::new_for_test("autoImportFileExcludePatterns2", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"./lib/main", "./lib/components/button/Button"}, &
}
