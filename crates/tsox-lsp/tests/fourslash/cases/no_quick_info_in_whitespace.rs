use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNotQuickInfoExists"]
#[test]
fn no_quick_info_in_whitespace() {
    let content = r#"class C {
/*1*/    private _mspointerupHandler(args) {
        if (args.button === 3) {
            return null; 
/*2*/        } else if (args.button === 4) {
/*3*/            return null;
        }
    }
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyNotQuickInfoExists"); // f.VerifyNotQuickInfoExists(t)
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifyNotQuickInfoExists"); // f.VerifyNotQuickInfoExists(t)
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifyNotQuickInfoExists"); // f.VerifyNotQuickInfoExists(t)
}
