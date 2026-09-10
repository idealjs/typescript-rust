use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_external_module_renamed_exports() {
    let content = r#"// @Filename: other.ts
export {};
// @Filename: index.ts
const c = 0;
export { c as yeahThisIsTotallyInScopeHuh };
export * as alsoNotInScope from "./other";

/**/"#;
    let mut s = Session::new_for_test("completionsExternalModuleRenamedExports", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
