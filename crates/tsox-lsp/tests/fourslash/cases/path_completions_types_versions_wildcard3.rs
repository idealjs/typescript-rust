use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_types_versions_wildcard3() {
    let content = r#"// @module: commonjs
// @resolveJsonModule: false
// @Filename: /node_modules/foo/package.json
{
  "types": "index.d.ts",
  "typesVersions": {
    ">=4.3.5": {
      "browser/*": ["dist/*"]
    }
  }
}
// @Filename: /node_modules/foo/nope.d.ts
export const nope = 0;
// @Filename: /node_modules/foo/dist/index.d.ts
export const index = 0;
// @Filename: /node_modules/foo/dist/blah.d.ts
export const blah = 0;
// @Filename: /node_modules/foo/dist/subfolder/one.d.ts
export const one = 0;
// @Filename: /a.ts
import { } from "foo//**/";"#;
    let mut s = Session::new_for_test("pathCompletionsTypesVersionsWildcard3", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some(""), &["browser", "nope", "dist"]);
    fourslash::insert(&mut s, "browser/");
    fourslash::verify_completions_unsorted_at(&mut s, None, &["blah", "index", "subfolder"]);
    fourslash::insert(&mut s, "subfolder/");
    fourslash::verify_completions_unsorted_at(&mut s, None, &["one"]);
}
