use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports_attributes() {
    let content = r#"import { A } from "./file";
import { type B } from "./file";
import { C } from "./file" with { type: "a" };
import { A as D } from "./file" with { type: "b" };
import { E } from "./file" with { type: "a" };
import { A as F } from "./file" with { type: "b" };

type G = A | B | C | D | E | F;"#;
    let mut s = Session::new_for_test("organizeImportsAttributes", content);
    // TODO: f.VerifyOrganizeImports(t,
}
