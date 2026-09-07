use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn comments_enums_fourslash() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"/** Enum of colors*/
enum /*1*/Colors {
    /** Fancy name for 'blue'*/
    /*2*/Cornflower,
    /** Fancy name for 'pink'*/
    /*3*/FancyPink
}
var /*4*/x = /*5*/Colors./*6*/Cornflower;
x = Colors./*7*/FancyPink;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "enum Colors", "Enum of colors")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(enum member) Colors.Cornflower = 0", "Fancy name for 'blue'")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(enum member) Colors.FancyPink = 1", "Fancy name for 'pink'")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "var x: Colors", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "enum Colors", "Enum of colors")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "(enum member) Colors.Cornflower = 0", "Fancy name for 'blue'")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "7", "(enum member) Colors.FancyPink = 1", "Fancy name for 'pink'")
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"6", "7"}, &fourslash.CompletionsExpectedList{
}
