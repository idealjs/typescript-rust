use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn add_interface_member_above_class() {
    let content = r#"
interface Intersection {
    /*insertHere*/
}
interface Scene { }
class /*className*/Sphere {
    constructor() {
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "className", "class Sphere", "")
    fourslash::go_to_marker(&mut s, "insertHere");
    fourslash::insert(&mut s, "ray: Ray;");
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "className", "class Sphere", "")
}
