use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_named_import_location() {
    let content = r#"// @Filename: file.ts
export var x = 10;
export var y = 10;
export { x as await, x as interface, x as unique };
export default class C {
}
// @Filename: a.ts
import { /*1*/ } from "./file";
import { x, /*2*/ } from "./file";
import { x, y, /*3*/ } from "./file";
import { x, y, await as await_, /*4*/ } from "./file";
import { x, y, await as await_, interface as interface_, /*5*/ } from "./file";
import { x, y, await as await_, interface as interface_, unique, /*6*/ } from "./file";"#;
    let mut s = Session::new_for_test("completionInNamedImportLocation", content);
    fourslash::go_to_file(&mut s, "a.ts");
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "5");
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "6");
    // TODO: f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
}
