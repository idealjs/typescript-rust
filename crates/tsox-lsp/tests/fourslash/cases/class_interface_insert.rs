use tsox_lsp::fourslash::{self, Session};

#[test]
fn class_interface_insert() {
    let content = r#"interface Intersection {
    dist: number;
}
/*interfaceGoesHere*/
class /*className*/Sphere {
    constructor(private center) {
    }
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "className", "class Sphere", "");
    fourslash::go_to_marker(&mut s, "interfaceGoesHere");
    fourslash::insert(
        &mut s,
        "\ninterface Surface {\n    reflect: () => number;\n}\n",
    );
    fourslash::verify_quick_info_at(&mut s, "className", "class Sphere", "");
}
