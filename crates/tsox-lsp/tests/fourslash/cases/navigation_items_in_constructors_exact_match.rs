use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWorkspaceSymbol"]
#[test]
fn navigation_items_in_constructors_exact_match() {
    let content = r#"// @noLib: true
class Test {
    private [|search1|]: number;
    constructor(public [|search2|]: boolean, readonly [|search3|]: string, search4: string) {
    }
}"#;
    let mut s = Session::new_for_test("navigationItemsInConstructorsExactMatch", content);
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
