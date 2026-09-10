use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlightsWithOptions"]
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
    let mut s = Session::new_for_test("findAllRefsForModule", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
    fourslash::unsupported("VerifyBaselineDocumentHighlightsWithOptions"); // f.VerifyBaselineDocumentHighlightsWithOptions(t, nil /*preferences*/, []string{"/b.ts", "/c/sub.js",
}
