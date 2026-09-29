use tsox_lsp::fourslash::Session;


#[test]
fn typedefinition01() {
    let content = r#"// @lib: es5
// @Filename: b.ts
import n = require('./a');
var x/*1*/ = new n.Foo();
// @Filename: a.ts
export class /*2*/Foo {}"#;
    let _s = Session::new_for_test("typedefinition01", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "1")
}
