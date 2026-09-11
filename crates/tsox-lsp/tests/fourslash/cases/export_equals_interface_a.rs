use tsox_lsp::fourslash::{self, Session};


#[test]
fn export_equals_interface_a() {
    let content = r#"// @Filename: exportEqualsInterface_A.ts
interface A {
    p1: number;
}
export = A;
/**/
var i: I1;
var n: number = i.p1;"#;
    let mut s = Session::new_for_test("exportEqualsInterfaceA", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "import I1 = require(\"exportEqualsInterface_A\");");
}
