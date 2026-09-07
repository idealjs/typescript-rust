use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_enum_members_accept_non_ascii_strings() {
    let content = r#"enum Demo {
    /*Emoji*/Emoji = '🍎',
    /*Hebrew*/Hebrew = 'תפוח',
    /*Chinese*/Chinese = '苹果',
    /*Japanese*/Japanese = 'りんご',
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Emoji", "(enum member) Demo.Emoji = \"🍎\"", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Hebrew", "(enum member) Demo.Hebrew = \"תפוח\"", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Chinese", "(enum member) Demo.Chinese = \"苹果\"", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Japanese", "(enum member) Demo.Japanese = \"りんご\"", "")
}
