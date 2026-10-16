use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_for_module() {
    let content = r#"// @allowJs: true
// @Filename: /a.ts
export const x = 0;
// @Filename: /b.ts
[|import { x } from "/*0*/[|{| "contextRangeIndex": 0 |}./a|]";|]
// @Filename: /c/sub.js
[|const a = require("/*1*/[|{| "contextRangeIndex": 2 |}../a|]");|]
// @Filename: /d.ts
 /// <reference path="/*2*/[|./a.ts|]" />"#;
    let _s = Session::new_for_test("findAllRefsForModule", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
    // TODO: f.VerifyBaselineDocumentHighlightsWithOptions(t, nil /*preferences*/, []string{"/b.ts", "/c/sub.js",
}
