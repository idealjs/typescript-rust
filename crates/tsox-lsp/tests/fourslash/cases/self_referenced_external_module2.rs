use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn self_referenced_external_module2() {
    let content = r#"// @Filename: app.ts
export import A = require('./app2');
export var I = 1;
A./*1*/Y;
A.B.A.B./*2*/I;
// @Filename: app2.ts
export import B = require('./app');
export var Y = 1;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var A.Y: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var I: number", "")
}
