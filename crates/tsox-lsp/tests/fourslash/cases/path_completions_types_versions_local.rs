use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("pathCompletionsTypesVersionsLocal", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["add"]);
}
