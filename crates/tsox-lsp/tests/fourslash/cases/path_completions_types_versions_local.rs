use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_types_versions_local() {
    let content = r#"// @Filename: /package.json
{
  "typesVersions": {
    "*": {
      "*": ["./src/*"]
    }
  }
}
// @Filename: /src/add.ts
export function add(a: number, b: number) { return a + b; }
// @Filename: /src/index.ts
import { add } from ".//**/";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
