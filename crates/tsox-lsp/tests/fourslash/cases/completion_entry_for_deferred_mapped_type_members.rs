use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_entry_for_deferred_mapped_type_members() {
    let content = r#"// @Filename: test.ts
interface A { a: A }
declare let a: A;
type Deep<T> = { [K in keyof T]: Deep<T[K]> }
declare function foo<T>(deep: Deep<T>): T;
const out = foo(a);
out./*1*/a
out.a./*2*/a
out.a.a./*3*/a"#;
    let mut s = Session::new_for_test("completionEntryForDeferredMappedTypeMembers", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
