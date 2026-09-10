use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports_attributes4() {
    let content = r#"import { A } from "./a" with { foo: "foo", bar: "bar" };
import { B } from "./a" with { bar: "bar", foo: "foo" };
import { D } from "./a" with { bar: "foo", foo: "bar" };
import { E } from "./a" with { foo: 'bar', bar: "foo" };
import { C } from "./a" with { foo: "bar", bar: "foo" };
import { F } from "./a" with { foo: "42" };
import { Y } from "./a" with { foo: 42 };
import { Z } from "./a" with { foo: "42" };

export type G = A | B | C | D | E | F | Y | Z;"#;
    let mut s = Session::new_for_test("organizeImportsAttributes4", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
