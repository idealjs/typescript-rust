use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_root_symbols() {
    let content = r#"interface I { /*0*/x: {}; }
interface J { /*1*/x: {}; }
declare const o: (I | J) & { /*2*/x: string };
o./*3*/x;"#;
    let _s = Session::new_for_test("findAllRefsRootSymbols", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
