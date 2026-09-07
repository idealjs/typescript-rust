use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn this_predicate_function_completions02() {
    let content = r#"interface Sundries {
    broken: boolean;
}

interface Supplies {
    spoiled: boolean;
}

interface Crate<T> {
    contents: T;
    isSundries(): this is Crate<Sundries>;
    isSupplies(): this is Crate<Supplies>;
    isPackedTight(): this is (this & {extraContents: T});
}
const crate: Crate<any>;
if (crate.isPackedTight()) {
    crate./*1*/;
}
if (crate.isSundries()) {
    crate.contents./*2*/;
    if (crate.isPackedTight()) {
        crate./*3*/;
    }
}
if (crate.isSupplies()) {
    crate.contents./*4*/;
    if (crate.isPackedTight()) {
        crate./*5*/;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "3", "5"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}
