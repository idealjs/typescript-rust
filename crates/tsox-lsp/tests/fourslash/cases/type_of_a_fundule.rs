use tsox_lsp::fourslash::{self, Session};

#[test]
fn type_of_a_fundule() {
    let content = r#"function m1() { return 1; }
namespace m1 { export var y = 2; }
function foo13() {
    return m1;
}
var /**/r13 = foo13();"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "var r13: typeof m1", "");
}
