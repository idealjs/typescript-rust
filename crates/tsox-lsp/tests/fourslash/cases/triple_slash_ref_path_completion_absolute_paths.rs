use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn triple_slash_ref_path_completion_absolute_paths() {
    let content = r#"// @Filename: /tests/cases/fourslash/tests/test0.ts
/// <reference path="/tests/cases/f/*0*/
// @Filename: /tests/cases/fourslash/tests/test1.ts
/// <reference path="/tests/cases/fourslash/*1*/
// @Filename: /tests/cases/fourslash/tests/test2.ts
/// <reference path="/tests/cases/fourslash//*2*/
// @Filename: /tests/cases/fourslash/f1.ts
/*f1*/
// @Filename: /tests/cases/fourslash/f2.tsx
/*f2*/
// @Filename: /tests/cases/fourslash/folder/f1.ts
/*subf1*/
// @Filename: /tests/cases/fourslash/f3.js
/*f3*/
// @Filename: /tests/cases/fourslash/f4.jsx
/*f4*/
// @Filename: /tests/cases/fourslash/e1.ts
/*e1*/
// @Filename: /tests/cases/fourslash/e2.js
/*e2*/"#;
    let mut s = Session::new_for_test("tripleSlashRefPathCompletionAbsolutePaths", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"0", "1"}, &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["e1.ts", "f1.ts", "f2.tsx", "folder", "tests"]);
}
