use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_symbol_members() {
    let content = r#"declare const Symbol: (s: string) => symbol;
const s = Symbol("s");
interface I { [s]: number };
declare const i: I;
i[|./*i*/|];

namespace N { export const s2 = Symbol("s2"); }
interface J { [N.s2]: number; }
declare const j: J;
j[|./*j*/|];"#;
    let mut s = Session::new_for_test("completionsSymbolMembers", content);
    fourslash::go_to_marker(&mut s, "i");
    // TODO: f.VerifyCompletions(t, "i", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "j");
    // TODO: f.VerifyCompletions(t, "j", &fourslash.CompletionsExpectedList{
}
