use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_checks1() {
    let content = r#"// @target: esnext
declare const obj: {
  a?: string;
  b: number;
};

if ("/*1*/" in obj) {}
if (((("/*2*/"))) in obj) {}
if ("/*3*/" in (((obj)))) {}
if (((("/*4*/"))) in (((obj)))) {}

type MyUnion = { missing: true } | { result: string };
declare const u: MyUnion;
if ("/*5*/" in u) {}

class Cls1 { foo = ''; #bar = 0; }
declare const c1: Cls1;
if ("/*6*/" in c1) {}

class Cls2 { foo = ''; private bar = 0; }
declare const c2: Cls2;
if ("/*7*/" in c2) {}"#;
    let mut s = Session::new_for_test("completionInChecks1", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2", "3", "4"}, &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("5"), &["missing", "result"]);
    fourslash::verify_completions_exact_at(&mut s, Some("6"), &["foo"]);
    fourslash::verify_completions_exact_at(&mut s, Some("7"), &["bar", "foo"]);
}
