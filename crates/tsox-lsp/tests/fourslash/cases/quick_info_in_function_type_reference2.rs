use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn quick_info_in_function_type_reference2() {
    let content = r#"class C<T> {
    map(fn: (/*1*/k: string, /*2*/value: T, context: any) => void, context: any) {
    }
}
var c: C<number>;
c.map(/*3*/"#;
    let mut s = Session::new_for_test("quickInfoInFunctionTypeReference2", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(parameter) k: string", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) value: T", "");
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "map(fn: (k: string, value: numb
}
