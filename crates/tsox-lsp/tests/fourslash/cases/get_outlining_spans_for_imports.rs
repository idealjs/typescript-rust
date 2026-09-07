use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn get_outlining_spans_for_imports() {
    let content = r#"[|import * as ns from "mod";

import d from "mod";
import { a, b, c } from "mod";

import r = require("mod");|]

// statement
var x = 0;

// another set of imports
[|import * as ns from "mod";
import d from "mod";
import { a, b, c } from "mod";
import r = require("mod");|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t, lsproto.FoldingRangeKindImports)
}
