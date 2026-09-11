use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_unique_symbol_import() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noLib: true
// @Filename: /globals.d.ts
declare const Symbol: () => symbol;
// @Filename: /a.ts
const privateSym = Symbol();
export const publicSym = Symbol();
export interface I {
    [privateSym]: number;
    [publicSym]: number;
    [defaultPublicSym]: number;
    n: number;
}
export const i: I;
// @Filename: /user.ts
import { i } from "./a";
i[|./**/|];"#;
    let mut s = Session::new_for_test("completionsUniqueSymbol_import", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
