use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_types_versions_wildcard6() {
    let content = r#"// @module: commonjs
// @Filename: /node_modules/foo/package.json
{
  "types": "index.d.ts",
  "typesVersions": {
    "*": {
      "bar/*": ["dist/*"],
      "exact-match": ["dist/index.d.ts"],
      "foo/*": ["dist/*"],
      "*": ["dist/*"]
    }
  }
}
// @Filename: /node_modules/foo/nope.d.ts
export const nope = 0;
// @Filename: /node_modules/foo/dist/index.d.ts
export const index = 0;
// @Filename: /node_modules/foo/dist/blah.d.ts
export const blah = 0;
// @Filename: /node_modules/foo/dist/foo/onlyInFooFolder.d.ts
export const foo = 0;
// @Filename: /node_modules/foo/dist/subfolder/one.d.ts
export const one = 0;
// @Filename: /a.ts
import { } from "foo//**/";"#;
    let mut s = Session::new_for_test("pathCompletionsTypesVersionsWildcard6", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some(""), &["bar", "exact-match", "foo", "blah", "index", "subfolder"]);
    fourslash::insert(&mut s, "foo/");
    fourslash::verify_completions_unsorted_at(&mut s, None, &["blah", "index", "foo", "subfolder"]);
    fourslash::insert(&mut s, "foo/");
    fourslash::verify_completions_unsorted_at(&mut s, None, &["onlyInFooFolder"]);
}
