use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_call_expression_tuples() {
    let content = r#"function fnTest(str: string, num: number) { }
declare function wrap<A extends any[], R>(fn: (...a: A) => R) : (...a: A) => R;
var fnWrapped = wrap(fnTest);
fnWrapped/*3*/(/*1*/'', /*2*/5);
function fnTestVariadic (str: string, ...num: number[]) { }
var fnVariadicWrapped = wrap(fnTestVariadic);
fnVariadicWrapped/*4*/(/*5*/'', /*6*/5);
function fnNoParams () { }
var fnNoParamsWrapped = wrap(fnNoParams);
fnNoParamsWrapped/*7*/(/*8*/);"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "3",
        "var fnWrapped: (str: string, num: number) => void",
        "",
    );
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "fnWrapped(str: string, num: num
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "num", ParameterSpan: "
    fourslash::verify_quick_info_at(
        &mut s,
        "4",
        "var fnVariadicWrapped: (str: string, ...num: number[]) => void",
        "",
    );
    fourslash::go_to_marker(&mut s, "5");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "fnVariadicWrapped(str: string, 
    fourslash::go_to_marker(&mut s, "6");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "num", ParameterSpan: "
    fourslash::verify_quick_info_at(&mut s, "7", "var fnNoParamsWrapped: () => void", "");
    fourslash::go_to_marker(&mut s, "8");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "fnNoParamsWrapped(): void", Par
}
