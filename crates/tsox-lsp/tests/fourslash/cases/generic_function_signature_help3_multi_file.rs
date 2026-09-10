use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn generic_function_signature_help3_multi_file() {
    let content = r#"// @Filename: genericFunctionSignatureHelp_0.ts
function foo1<T>(x: number, callback: (y1: T) => number) { }
// @Filename: genericFunctionSignatureHelp_1.ts
function foo2<T>(x: number, callback: (y2: T) => number) { }
// @Filename: genericFunctionSignatureHelp_2.ts
function foo3<T>(x: number, callback: (y3: T) => number) { }
// @Filename: genericFunctionSignatureHelp_3.ts
function foo4<T>(x: number, callback: (y4: T) => number) { }
// @Filename: genericFunctionSignatureHelp_4.ts
function foo5<T>(x: number, callback: (y5: T) => number) { }
// @Filename: genericFunctionSignatureHelp_5.ts
function foo6<T>(x: number, callback: (y6: T) => number) { }
// @Filename: genericFunctionSignatureHelp_6.ts
function foo7<T>(x: number, callback: (y7: T) => number) { }
// @Filename: genericFunctionSignatureHelp_7.ts
foo1(/*1*/               // signature help shows y as T
foo2(1,/*2*/             // signature help shows y as {}
foo3(1, (/*3*/           // signature help shows y as T
foo4<string>(1,/*4*/     // signature help shows y as string
foo5<string>(1, (/*5*/   // signature help shows y as T
foo6(1, </*6*/           // signature help shows y as {}
foo7(1, <string>(/*7*/   // signature help shows y as T"#;
    let mut s = Session::new_for_test("genericFunctionSignatureHelp3MultiFile", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo1(x: number, callback: (y1: 
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo2(x: number, callback: (y2: 
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "callback(y3: unknown): number"}
    fourslash::go_to_marker(&mut s, "4");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo4(x: number, callback: (y4: 
    fourslash::go_to_marker(&mut s, "5");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "callback(y5: string): number"})
    fourslash::go_to_marker(&mut s, "6");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo6(x: number, callback: (y6: 
    fourslash::insert(&mut s, "string>(null,null);");
    fourslash::go_to_marker(&mut s, "7");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo7(x: number, callback: (y7: 
}
