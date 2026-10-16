use tsox_lsp::fourslash::Session;


#[test]
fn definition() {
    let content = r#"// @Filename: b.ts
import n = require([|'./a/*1*/'|]);
var x = new n.Foo();
// @Filename: a.ts
 /*2*/export class Foo {}"#;
    let _s = Session::new_for_test("definition", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
