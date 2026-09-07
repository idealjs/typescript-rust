use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_of_constructor_multiple_files() {
    let content = r#"// @Filename: f.ts
class A {
    /*aCtr*/constructor(s: string) {}
}
class B extends A { }
export { A, B };
// @Filename: a.ts
import { A as A1 } from "./f";
const a1 = new A1("a1");
export default class extends A1 { }
export { B as B1 } from "./f";
// @Filename: b.ts
import B, { B1 } from "./a";
const d = new B("b");
const d1 = new B1("b1");"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "aCtr")
}
