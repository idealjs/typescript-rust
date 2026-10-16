use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_method_default_exported() {
    let content = r#"// @filename: other.ts
export default class Other {}
// @filename: base.ts
import Other from "./other";
export class Base {
    foo(): Other {
        throw new Error("");
    }
}
// @filename: derived.ts
import { Base } from "./base";
export class Derived extends Base {
    /**/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethodDefaultExported", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
