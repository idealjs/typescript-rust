use tsox_lsp::fourslash::{self, Session};


#[test]
fn called_unions_of_dissimilar_tyeshave_good_display() {
    let content = r#"declare const callableThing1:
    | ((o1: {x: number}) => void)
    | ((o1: {y: number}) => void)
    ;

callableThing1(/*1*/);

declare const callableThing2:
    | ((o1: {x: number}) => void)
    | ((o2: {y: number}) => void)
    ;

callableThing2(/*2*/);

declare const callableThing3:
    | ((o1: {x: number}) => void)
    | ((o2: {y: number}) => void)
    | ((o3: {z: number}) => void)
    | ((o4: {u: number}) => void)
    | ((o5: {v: number}) => void)
    ;

callableThing3(/*3*/);

declare const callableThing4:
    | ((o1: {x: number}) => void)
    | ((o2: {y: number}) => void)
    | ((o3: {z: number}) => void)
    | ((o4: {u: number}) => void)
    | ((o5: {v: number}) => void)
    | ((o6: {w: number}) => void)
    ;

callableThing4(/*4*/);

declare const callableThing5: 
    | (<U>(a1: U) => void)
    | (() => void) 
    ;

callableThing5(/*5*/1)
"#;
    let mut s = Session::new_for_test("calledUnionsOfDissimilarTyeshaveGoodDisplay", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "callableThing1(o1: { x: number;
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "callableThing2(arg0: { x: numbe
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "callableThing3(arg0: { x: numbe
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "callableThing4(arg0: { x: numbe
    fourslash::go_to_marker(&mut s, "5");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "callableThing5(a1: number): voi
}
