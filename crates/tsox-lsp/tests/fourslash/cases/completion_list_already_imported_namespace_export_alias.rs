use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: result := f.VerifyCompletions(t, '', &fourslash.CompletionsE"]
#[test]
fn completion_list_already_imported_namespace_export_alias() {
    let content = r#"// @module: node18
// @Filename: /values.ts
export const A = 1;
export const B = 2;

// @Filename: /namespace.ts
import * as Group from "./values.js";
type Group = (typeof Group)[keyof typeof Group];
export { Group };

// @Filename: /index.ts
import { Group } from "./namespace.js";

console.log(Grou/**/);"#;
    let mut s = Session::new(content);
    // TODO: result := f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: result.AndHasNoCodeAction(t, &fourslash.CompletionsExpectedCodeAction{
}
