use tsox_lsp::fourslash::{self, Session};


#[test]
fn alias_to_var_used_as_type() {
    let content = r#"/**/
namespace A {
export var X;
import Z = A.X;
var v: Z;
}"#;
    let mut s = Session::new_for_test("aliasToVarUsedAsType", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, " ");
}
