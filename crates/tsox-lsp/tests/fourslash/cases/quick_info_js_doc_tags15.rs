use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_js_doc_tags15() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @filename: /a.js
/**
 * @callback Bar
 * @param {string} name
 * @returns {string}
 */

/**
 * @typedef Foo
 * @property {Bar} getName
 */
export const foo = 1;
// @filename: /b.js
import * as _a from "./a.js";
/**
 * @implements {_a.Foo/*1*/}
 */
class C1 { }

/**
 * @extends {_a.Foo/*2*/}
 */
class C2 { }

/**
 * @augments {_a.Foo/*3*/}
 */
class C3 { }"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/b.js");
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
