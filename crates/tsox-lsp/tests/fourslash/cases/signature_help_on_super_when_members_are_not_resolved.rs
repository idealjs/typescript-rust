use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_on_super_when_members_are_not_resolved() {
    let content = r#"class A { }
class B extends A { constructor(public x: string) { } }
class C extends B {
    constructor() {
        /*1*/
     }
}"#;
    let mut s = Session::new_for_test("signatureHelpOnSuperWhenMembersAreNotResolved", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "super(");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "B(x: string): B"})
}
