use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_enum_members_accept_non_ascii_strings() {
    let content = r#"enum Demo {
    /*Emoji*/Emoji = '🍎',
    /*Hebrew*/Hebrew = 'תפוח',
    /*Chinese*/Chinese = '苹果',
    /*Japanese*/Japanese = 'りんご',
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "Emoji", "(enum member) Demo.Emoji = \"🍎\"", "");
    fourslash::verify_quick_info_at(&mut s, "Hebrew", "(enum member) Demo.Hebrew = \"תפוח\"", "");
    fourslash::verify_quick_info_at(
        &mut s,
        "Chinese",
        "(enum member) Demo.Chinese = \"苹果\"",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "Japanese",
        "(enum member) Demo.Japanese = \"りんご\"",
        "",
    );
}
