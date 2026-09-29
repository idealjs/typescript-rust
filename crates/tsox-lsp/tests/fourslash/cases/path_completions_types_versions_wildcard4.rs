use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_types_versions_wildcard4() {
    let content = r#"// @module: commonjs
// @resolveJsonModule: false
// @Filename: /node_modules/foo/package.json
{
  "types": "index.d.ts",
  "typesVersions": {
    ">=4.3.5": {
      "component-*": ["cjs/components/*"]
    }
  }
}
// @Filename: /node_modules/foo/nope.d.ts
export const nope = 0;
// @Filename: /node_modules/foo/cjs/components/index.d.ts
export const index = 0;
// @Filename: /node_modules/foo/cjs/components/blah.d.ts
export const blah = 0;
// @Filename: /node_modules/foo/cjs/components/subfolder/one.d.ts
export const one = 0;
// @Filename: /a.ts
import { } from "foo//**/";"#;
    let mut s = Session::new_for_test("pathCompletionsTypesVersionsWildcard4", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some(""), &["component-blah", "component-index", "component-subfolder", "nope", "cjs"]);
    fourslash::insert(&mut s, "component-subfolder/");
    fourslash::verify_completions_unsorted_at(&mut s, None, &["one"]);
}
