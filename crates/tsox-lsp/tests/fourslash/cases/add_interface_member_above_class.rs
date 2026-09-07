use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "className", "class Sphere", "");
    fourslash::go_to_marker(&mut s, "insertHere");
    fourslash::insert(&mut s, "ray: Ray;");
    fourslash::verify_quick_info_at(&mut s, "className", "class Sphere", "");
}
