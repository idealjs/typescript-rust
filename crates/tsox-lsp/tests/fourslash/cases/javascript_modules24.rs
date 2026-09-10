use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn javascript_modules24() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: mod.ts
function foo() { return 42; }
namespace foo {
  export function bar (a: string) { return a; }
}
export = foo;
// @Filename: app.ts
import * as foo from "./mod"
foo/*1*/();
foo.bar(/*2*/"test");"#;
    let mut s = Session::new_for_test("javascriptModules24", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyErrorExistsBeforeMarker"); // f.VerifyErrorExistsBeforeMarker(t, "1")
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(alias) function foo(): number\n(alias) namespace foo\nimport foo", "")
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{})
}
