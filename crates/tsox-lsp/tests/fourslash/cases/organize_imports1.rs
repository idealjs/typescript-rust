use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports1() {
    let content = r#"import {
    d, d as D,
    c,
    c as C, b,
    b as B, a
} from './foo';
import {
    h, h as H,
    g,
    g as G, f,
    f as F, e
} from './foo';

console.log(a, B, b, c, C, d, D);
console.log(e, f, F, g, G, H, h);"#;
    let mut s = Session::new_for_test("organizeImports1", content);
    fourslash::unsupported("VerifyOrganizeImportsWithRequestKind"); // f.VerifyOrganizeImportsWithRequestKind(t,
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
