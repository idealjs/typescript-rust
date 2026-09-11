use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_paths_relative_json_module() {
    let content = r#"// @moduleResolution: bundler
// @resolveJsonModule: true
// @Filename: /project/test.json
not read
// @Filename: /project/index.ts
import { } from ".//**/";"#;
    let mut s = Session::new_for_test("completionsPathsRelativeJsonModule", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
