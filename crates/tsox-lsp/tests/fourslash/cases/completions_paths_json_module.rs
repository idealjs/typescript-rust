use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_paths_json_module() {
    let content = r#"// @moduleResolution: bundler
// @resolveJsonModule: true
// @Filename: /project/node_modules/test.json
not read
// @Filename: /project/index.ts
import { } from "/**/";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
