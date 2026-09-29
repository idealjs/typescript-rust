use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("quickinfoVerbosityClassInterfaceMerge", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{
}
