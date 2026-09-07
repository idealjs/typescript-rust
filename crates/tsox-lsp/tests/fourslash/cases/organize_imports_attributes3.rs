use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports_attributes3() {
    let content = r#"import { A } from "./a";
import { C } from "./a" with {      type: "a" };
import { Z } from "./z";
import { A as D } from "./a" with    { type: "b" };
import { E } from "./a" with { type: /* comment*/ "a"              };
import { F } from "./a" with     {type: "a" };
import { Y } from "./a"   with{ type: "b" /* comment*/};
import { B } from "./a";

export type G = A | B | C | D | E | F | Y | Z;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
