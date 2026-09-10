use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_class_interface_merge() {
    let content = r#"
declare class Foo/*1*/ {
    x: number;
}
declare interface Foo {
    y: string;
}
const f: Foo/*2*/ = { x: 1, y: "hello" };
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityClassInterfaceMerge", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{
}
