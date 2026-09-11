use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports_attributes2() {
    let content = r#"import { A } from "./a";
import { C } from "./a" with { type: "a" };
import { Z } from "./z";
import { A as D } from "./a" with { type: "b" };
import { E } from "./a" with { type: "a" };
import { F } from "./a" with { type: "a" };
import { B } from "./a";

export type G = A | B | C | D | E | F | Z;"#;
    let mut s = Session::new_for_test("organizeImportsAttributes2", content);
    // TODO: f.VerifyOrganizeImports(t,
}
