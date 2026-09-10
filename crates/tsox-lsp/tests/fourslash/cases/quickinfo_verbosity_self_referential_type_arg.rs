use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_self_referential_type_arg() {
    let content = r#"type ContainerChild = Container;
interface Container<C = ContainerChild> {
    parent: Container;
}
declare const x: Container;
x/*1*/;"#;
    let mut s = Session::new_for_test("quickinfoVerbositySelfReferentialTypeArg", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {3}})
}
