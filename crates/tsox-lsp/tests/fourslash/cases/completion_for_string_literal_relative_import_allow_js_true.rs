use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_string_literal_relative_import_allow_js_true() {
    let content = r#"// @allowJs: true
// @Filename: test0.ts
import * as foo1 from ".//*import_as0*/
import * as foo2 from "./f/*import_as1*/
import foo3 = require(".//*import_equals0*/
import foo4 = require("./f/*import_equals1*/
var foo5 = require(".//*require0*/
var foo6 = require("./f/*require1*/
// @Filename: f1.ts

// @Filename: f2.js

// @Filename: f3.d.ts

// @Filename: f4.tsx

// @Filename: f5.js

// @Filename: f6.jsx

// @Filename: g1.ts

// @Filename: g2.js
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
