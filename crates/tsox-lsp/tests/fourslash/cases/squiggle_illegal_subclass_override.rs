use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyErrorExistsBetweenMarkers"]
#[test]
fn squiggle_illegal_subclass_override() {
    let content = r#"// @strict: false
class Foo {
    public x: number;
}

class Bar extends Foo {
    public /*1*/x/*2*/: string = 'hi';
}"#;
    let mut s = Session::new_for_test("squiggleIllegalSubclassOverride", content);
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
