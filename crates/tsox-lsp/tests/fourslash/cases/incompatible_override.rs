use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyErrorExistsBetweenMarkers"]
#[test]
fn incompatible_override() {
    let content = r#"// @strict: false
class Foo { xyz: string; }
class Bar extends Foo { /*1*/xyz/*2*/: number = 1; }
class Baz extends Foo { public /*3*/xyz/*4*/: number = 2; }
class /*5*/Baf/*6*/ extends Foo {
   constructor(public xyz: number) {
      super();
   }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "3", "4")
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "5", "6")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 3);
}
