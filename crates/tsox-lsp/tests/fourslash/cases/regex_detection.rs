use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNotQuickInfoExists"]
#[test]
fn regex_detection() {
    let content = r#" /*1*/15 / /*2*/Math.min(61 / /*3*/42, 32 / 15) / /*4*/15;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyNotQuickInfoExists"); // f.VerifyNotQuickInfoExists(t)
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "var Math: Math", "An intrinsic object that provides basic mathematics functi
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifyNotQuickInfoExists"); // f.VerifyNotQuickInfoExists(t)
    fourslash::go_to_marker(&mut s, "4");
    fourslash::unsupported("VerifyNotQuickInfoExists"); // f.VerifyNotQuickInfoExists(t)
}
