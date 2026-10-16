use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn comments_import_declaration() {
    let content = r#"// @Filename: commentsImportDeclaration_file0.ts
/** NamespaceComment*/
export namespace m/*2*/1 {
    /** b's comment*/
    export var b: number;
    /** m2 comments*/
    export namespace m2 {
        /** class comment;*/
        export class c {
        };
        /** i*/
        export var i: c;;
    }
    /** exported function*/
    export function fooExport(): number;
}
// @Filename: commentsImportDeclaration_file1.ts
///<reference path='commentsImportDeclaration_file0.ts'/>
/** Import declaration*/
import /*3*/extMod = require("./commentsImportDeclaration_file0/*4*/");
extMod./*6*/m1./*7*/fooEx/*8q*/port(/*8*/);
var new/*9*/Var = new extMod.m1.m2./*10*/c();"#;
    let mut s = Session::new_for_test("commentsImportDeclaration", content);
    fourslash::verify_quick_info_at(&mut s, "2", "namespace m1", "NamespaceComment");
    fourslash::verify_quick_info_at(&mut s, "3", "import extMod = require(\"./commentsImportDeclaration_file0\")", "Import declaration");
    fourslash::go_to_marker(&mut s, "6");
    // TODO: f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "7");
    // TODO: f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "8");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "exported function"})
    fourslash::verify_quick_info_at(&mut s, "8q", "function extMod.m1.fooExport(): number", "exported function");
    fourslash::verify_quick_info_at(&mut s, "9", "var newVar: extMod.m1.m2.c", "");
    fourslash::go_to_marker(&mut s, "10");
    // TODO: f.VerifyCompletions(t, "10", &fourslash.CompletionsExpectedList{
}
