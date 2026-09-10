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
    let mut s = Session::new_for_test("commentsEnumsFourslash", content);
    fourslash::verify_quick_info_at(&mut s, "1", "enum Colors", "Enum of colors");
    fourslash::verify_quick_info_at(&mut s, "2", "(enum member) Colors.Cornflower = 0", "Fancy name for 'blue'");
    fourslash::verify_quick_info_at(&mut s, "3", "(enum member) Colors.FancyPink = 1", "Fancy name for 'pink'");
    fourslash::verify_quick_info_at(&mut s, "4", "var x: Colors", "");
    fourslash::verify_quick_info_at(&mut s, "5", "enum Colors", "Enum of colors");
    fourslash::verify_quick_info_at(&mut s, "6", "(enum member) Colors.Cornflower = 0", "Fancy name for 'blue'");
    fourslash::verify_quick_info_at(&mut s, "7", "(enum member) Colors.FancyPink = 1", "Fancy name for 'pink'");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"6", "7"}, &fourslash.CompletionsExpectedList{
}
