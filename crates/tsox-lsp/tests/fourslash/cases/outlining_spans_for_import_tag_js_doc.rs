use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn outlining_spans_for_import_tag_js_doc() {
    let content = r#"
// @allowJs: true
// @checkJs: true	
// @Filename: /a.js
[|/**
 * @import {b} from "./b.js";
 * @import {c} from "./c.js";
 */|]

 [|/**
 * @import {d} from "./d.js";
 */|]

"#;
    let mut s = Session::new_for_test("outliningSpansForImportTagJSDoc", content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
