use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("thisPredicateFunctionCompletions02", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "3", "5"}, &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["broken"]);
    fourslash::verify_completions_exact_at(&mut s, Some("4"), &["spoiled"]);
}
