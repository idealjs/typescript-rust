use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn typedefinition01() {
    let content = r#"// @lib: es5
// @Filename: b.ts
import n = require('./a');
var x/*1*/ = new n.Foo();
// @Filename: a.ts
export class /*2*/Foo {}"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineGoToTypeDefinition"); // f.VerifyBaselineGoToTypeDefinition(t, "1")
}
