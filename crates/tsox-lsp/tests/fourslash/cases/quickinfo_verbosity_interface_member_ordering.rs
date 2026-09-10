use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_interface_member_ordering() {
    let content = r#"
interface Callable/*1*/ {
    (x: string): boolean;
    new (x: string): Callable;
    [key: string]: any;
    name: string;
}
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityInterfaceMemberOrdering", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
