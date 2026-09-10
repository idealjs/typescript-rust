use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn quick_info_on_narrowed_type() {
    let content = r#"// @strictNullChecks: true
function foo(strOrNum: string | number) {
    if (typeof /*1*/strOrNum === "number") {
        return /*2*/strOrNum;
    }
    else {
        return /*3*/strOrNum.length;
    }
}
function bar() {
   let s: string | undefined;
   /*4*/s;
   /*5*/s = "abc";
   /*6*/s;
}
class Foo {
    #privateProperty: string[] | null;
    constructor() {
        this.#privateProperty = null;
    }
    testMethod() {
        if (this.#privateProperty === null)
            return;
        this./*7*/#privateProperty;
    }
}"#;
    let mut s = Session::new_for_test("quickInfoOnNarrowedType", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(parameter) strOrNum: string | number", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) strOrNum: number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(parameter) strOrNum: string", "");
    fourslash::verify_quick_info_at(&mut s, "4", "let s: string | undefined", "");
    fourslash::verify_quick_info_at(&mut s, "5", "let s: string | undefined", "");
    fourslash::verify_quick_info_at(&mut s, "6", "let s: string", "");
    fourslash::verify_quick_info_at(&mut s, "7", "(property) Foo.#privateProperty: string[]", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"4", "5"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
}
