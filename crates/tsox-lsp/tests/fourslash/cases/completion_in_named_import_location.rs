use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
}
