use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_imports() {
    let content = r#"import d1 from "a";

import { a } from "a";

import { b as B } from "a" 

import d2, { c, d as D } from "a" 

import e = require("a");

import * as ns from "a";"#;
    let mut s = Session::new_for_test("navigationBarItemsImports", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
