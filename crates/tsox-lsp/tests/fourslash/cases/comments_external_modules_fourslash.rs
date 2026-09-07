use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn comments_external_modules_fourslash() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: commentsExternalModules_file0.ts
/** Namespace comment*/
export namespace m/*1*/1 {
    /** b's comment*/
    export var b: number;
    /** foo's comment*/
    function foo() {
        return /*2*/b;
    }
    /** m2 comments*/
    export namespace m2 {
        /** class comment;*/
        export class c {
        };
        /** i*/
        export var i = new c();
    }
    /** exported function*/
    export function fooExport() {
        return f/*3q*/oo(/*3*/);
    }
}
/*4*/m1./*5*/fooEx/*6q*/port(/*6*/);
var my/*7*/var = new m1.m2./*8*/c();
// @Filename: commentsExternalModules_file1.ts
/**This is on import declaration*/
import ex/*9*/tMod = require("./commentsExternalModules_file0");
/*10*/extMod./*11*/m1./*12*/fooExp/*13q*/ort(/*13*/);
var new/*14*/Var = new extMod.m1.m2./*15*/c();"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "commentsExternalModules_file0.ts");
    fourslash::verify_quick_info_at(&mut s, "1", "namespace m1", "Namespace comment");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "foo's comment"})
    fourslash::verify_quick_info_at(&mut s, "3q", "function foo(): number", "foo's comment");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "6");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "exported function"})
    fourslash::verify_quick_info_at(
        &mut s,
        "6q",
        "function m1.fooExport(): number",
        "exported function",
    );
    fourslash::verify_quick_info_at(&mut s, "7", "var myvar: m1.m2.c", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "8", &fourslash.CompletionsExpectedList{
    fourslash::go_to_file(&mut s, "commentsExternalModules_file1.ts");
    fourslash::verify_quick_info_at(
        &mut s,
        "9",
        "import extMod = require(\"./commentsExternalModules_file0\")",
        "This is on import declaration",
    );
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "10", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "11", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "12", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "13");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "exported function"})
    fourslash::verify_quick_info_at(
        &mut s,
        "13q",
        "function extMod.m1.fooExport(): number",
        "exported function",
    );
    fourslash::verify_quick_info_at(&mut s, "14", "var newVar: extMod.m1.m2.c", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "15", &fourslash.CompletionsExpectedList{
}
