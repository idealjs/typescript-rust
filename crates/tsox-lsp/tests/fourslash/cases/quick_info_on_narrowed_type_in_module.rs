use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_narrowed_type_in_module() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
var strOrNum: string | number;
namespace m {
    var nonExportedStrOrNum: string | number;
    export var exportedStrOrNum: string | number;
    var num: number;
    var str: string;
    if (typeof /*1*/nonExportedStrOrNum === "number") {
        num = /*2*/nonExportedStrOrNum;
    }
    else {
        str = /*3*/nonExportedStrOrNum.length;
    }
    if (typeof /*4*/exportedStrOrNum === "number") {
        strOrNum = /*5*/exportedStrOrNum;
    }
    else {
        strOrNum = /*6*/exportedStrOrNum;
    }
}
if (typeof m./*7*/exportedStrOrNum === "number") {
    strOrNum = m./*8*/exportedStrOrNum;
}
else {
    strOrNum = m./*9*/exportedStrOrNum;
}"#;
    let mut s = Session::new_for_test("quickInfoOnNarrowedTypeInModule", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var nonExportedStrOrNum: string | number", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var nonExportedStrOrNum: number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var nonExportedStrOrNum: string", "");
    fourslash::verify_quick_info_at(&mut s, "4", "var m.exportedStrOrNum: string | number", "");
    fourslash::verify_quick_info_at(&mut s, "5", "var m.exportedStrOrNum: number", "");
    fourslash::verify_quick_info_at(&mut s, "6", "var m.exportedStrOrNum: string", "");
    fourslash::verify_quick_info_at(&mut s, "7", "var m.exportedStrOrNum: string | number", "");
    fourslash::verify_quick_info_at(&mut s, "8", "var m.exportedStrOrNum: number", "");
    fourslash::verify_quick_info_at(&mut s, "9", "var m.exportedStrOrNum: string", "");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "8", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "9", &fourslash.CompletionsExpectedList{
}
