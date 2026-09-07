use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // Regression test for https://github.com/microsoft/TypeScri"]
#[test]
fn completions_from_untitled_file() {
    // TODO: // Test that completions work in untitled files without crashing.
    // TODO: // Regression test for https://github.com/microsoft/TypeScript/tsc/issues/2550
    let content = r#"// @filename: /home/src/project/utils.ts
export function helper() {}

// @filename: ^/untitled/ts-nul-authority/Untitled-1.ts
/**/"#;
    let mut s = Session::new(content);
    // TODO: // Request completions - this should not crash
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
