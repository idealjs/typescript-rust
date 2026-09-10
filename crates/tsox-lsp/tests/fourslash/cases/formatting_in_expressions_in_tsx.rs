use tsox_lsp::fourslash::{self, Session};


#[ignore = "needs live LSP session"]
#[test]
fn formatting_in_expressions_in_tsx() {
    let content = r#"// @Filename: test.tsx
import * as React from "react";
<div
    autoComplete={(function () {
return true/*1*/
    })() }
    >
</div>"#;
    let mut s = Session::new_for_test("formattingInExpressionsInTsx", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"        return true;"#);
}
