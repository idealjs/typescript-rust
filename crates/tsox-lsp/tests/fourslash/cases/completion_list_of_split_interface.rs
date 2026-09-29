use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_of_split_interface() {
    let content = r#"interface A {
    a: number;
}
interface I extends A {
    i1: number;
}
interface I1 extends A {
    i11: number;
}
interface B {
    b: number;
}
interface B1 {
    b1: number;
}
interface I extends B {
    i2: number;
}
interface I1 extends B, B1 {
    i12: number;
}
interface C {
    c: number;
}
interface I extends C {
    i3: number;
}
var ci: I;
ci./*1*/b;
var ci1: I1;
ci1./*2*/b;"#;
    let mut s = Session::new_for_test("completionListOfSplitInterface", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["i1", "i2", "i3", "a", "b", "c"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("2"), &["i11", "i12", "a", "b", "b1"]);
}
