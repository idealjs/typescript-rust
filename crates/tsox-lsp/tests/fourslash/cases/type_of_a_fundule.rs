use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn type_of_a_fundule() {
    let content = r#"function m1() { return 1; }
namespace m1 { export var y = 2; }
function foo13() {
    return m1;
}
var /**/r13 = foo13();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var r13: typeof m1", "")
}
