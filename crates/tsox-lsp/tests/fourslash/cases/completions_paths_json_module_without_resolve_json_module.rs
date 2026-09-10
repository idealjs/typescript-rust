use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_paths_json_module_without_resolve_json_module() {
    let content = r#"// @resolveJsonModule: false
// @Filename: /project/test.json
not read
// @Filename: /project/index.ts
import { } from ".//**/";"#;
    let mut s = Session::new_for_test("completionsPathsJsonModuleWithoutResolveJsonModule", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
