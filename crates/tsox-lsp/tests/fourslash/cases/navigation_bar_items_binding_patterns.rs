use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_binding_patterns() {
    let content = r#"'use strict'
var foo, {}
var bar, []
let foo1, {a, b}
const bar1, [c, d]
var {e, x: [f, g]} = {a:1, x:[]};
var { h: i = function j() {} } = obj;"#;
    let mut s = Session::new_for_test("navigationBarItemsBindingPatterns", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
