use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn consistent_contextual_type_errors_after_edits() {
    let content = r#"// @strict: false
class A {
    foo: string;
}
class C {
    foo: string;
}
var xs /*1*/ = [(x: A) => { return x.foo; }, (x: C) => { return x.foo; }];
xs.forEach(y => y(new /*2*/A()));"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 0)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ": {}[]");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 1)
    fourslash::insert(&mut s, "C");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
}
