use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_unresolved_symbols2() {
    let content = r#"import { /*a0*/Bar } from "does-not-exist";

let a: /*a1*/Bar;
let b: /*a2*/Bar<string>;
let c: /*a3*/Bar<string, number>;
let d: /*a4*/Bar./*b0*/X;
let e: /*a5*/Bar./*b1*/X<string>;
let f: /*a6*/Bar./*c0*/X./*d0*/Y;"#;
    let _s = Session::new_for_test("findAllRefsUnresolvedSymbols2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "a0", "a1", "a2", "a3", "a4", "a5", "a6", "b0", "b1", "c0", "d0
}
