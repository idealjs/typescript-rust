use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_paths_json_module() {
    let content = r#"// @moduleResolution: bundler
// @resolveJsonModule: true
// @Filename: /project/node_modules/test.json
not read
// @Filename: /project/index.ts
import { } from "/**/";"#;
    let mut s = Session::new_for_test("completionsPathsJsonModule", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
