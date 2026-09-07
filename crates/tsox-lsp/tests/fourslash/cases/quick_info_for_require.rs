use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoIs"]
#[test]
fn quick_info_for_require() {
    let content = r#"//@Filename: AA/BB.ts
export class a{}
//@Filename: quickInfoForRequire_input.ts
import a = require("./AA/B/*1*/B");
import b = require(` + "`" + `./AA/B/*2*/B` + "`" + `);"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "module a", "")
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "module a", "")
}
