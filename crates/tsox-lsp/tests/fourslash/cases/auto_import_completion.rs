use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_completion1() {
    let content = r#"// @Filename: a.ts
export const someVar = 10;

// @Filename: b.ts
export const anotherVar = 10;

// @Filename: c.ts
import {someVar} from "./a.ts";
someVar;
a/**/
"#;
    let mut s = Session::new_for_test("autoImportCompletion1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

#[test]
fn auto_import_completion2() {
    let content = r#"// @Filename: a.ts
export const someVar = 10;
export const anotherVar = 10;

// @Filename: c.ts
import {someVar} from "./a.ts";
someVar;
a/**/
"#;
    let mut s = Session::new_for_test("autoImportCompletion2", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}

#[test]
fn auto_import_completion3() {
    let content = r#"// @Filename: a.ts
export const aa = "asdf";
export const someVar = 10;
export const bb = 10;

// @Filename: c.ts
import { aa, someVar } from "./a.ts";
someVar;
b/**/
"#;
    let mut s = Session::new_for_test("autoImportCompletion3", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}
