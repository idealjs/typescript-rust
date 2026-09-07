use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_external_module_name3() {
    let content = r#"// @Filename: b.ts
import n = require([|'e/*1*/'|]);
var x = new n.Foo();
// @Filename: a.ts
declare module /*2*/"e" {
    class Foo { }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
