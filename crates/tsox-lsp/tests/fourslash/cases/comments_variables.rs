use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn comments_variables() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"/** This is my variable*/
var myV/*1*/ariable = 10;
/*2*/
/** d variable*/
var d = 10;
myVariable = d;
/*3*/
/** foos comment*/
function foo() {
}
/** fooVar comment*/
var foo/*12*/Var: () => void;
/*4*/
f/*5q*/oo(/*5*/);
fo/*6q*/oVar(/*6*/);
fo/*13*/oVar = f/*14*/oo;
/*7*/
f/*8q*/oo(/*8*/);
foo/*9q*/Var(/*9*/);
var fooVarVar = /*9aq*/fooVar;
/**class comment*/
class c {
    /** constructor comment*/
    constructor() {
    }
}
/**instance comment*/
var i = new c();
/*10*/
/** interface comments*/
interface i1 {
}
/**interface instance comments*/
var i1_i: i1;
/*11*/
function foo2(a: number): void;
function foo2(b: string): void;
function foo2(aOrb) {
}
var x = fo/*15*/o2;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var myVariable: number", "This is my variable")
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "5");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "foos comment"})
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5q", "function foo(): void", "foos comment")
    fourslash::go_to_marker(&mut s, "6");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "fooVar comment"})
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6q", "var fooVar: () => void", "fooVar comment")
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "8");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "foos comment"})
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "8q", "function foo(): void", "foos comment")
    fourslash::go_to_marker(&mut s, "9");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "fooVar comment"})
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "9q", "var fooVar: () => void", "fooVar comment")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "9aq", "var fooVar: () => void", "fooVar comment")
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "10", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "11", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "12", "var fooVar: () => void", "fooVar comment")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "13", "var fooVar: () => void", "fooVar comment")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "14", "function foo(): void", "foos comment")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "15", "function foo2(a: number): void (+1 overload)", "")
}
