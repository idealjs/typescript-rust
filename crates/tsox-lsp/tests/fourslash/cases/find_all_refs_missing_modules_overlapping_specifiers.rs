use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_missing_modules_overlapping_specifiers() {
    let content = r#"// https://github.com/microsoft/TypeScript/issues/5551
import { resolve/*0*/ as resolveUrl } from "idontcare";
import { resolve/*1*/ } from "whatever";"#;
    let _s = Session::new_for_test("findAllRefsMissingModulesOverlappingSpecifiers", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1")
}
