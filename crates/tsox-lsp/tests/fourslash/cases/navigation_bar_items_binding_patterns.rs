use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_binding_patterns() {
    let content = r#"'use strict'
var foo, {}
var bar, []
let foo1, {a, b}
const bar1, [c, d]
var {e, x: [f, g]} = {a:1, x:[]};
var { h: i = function j() {} } = obj;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
