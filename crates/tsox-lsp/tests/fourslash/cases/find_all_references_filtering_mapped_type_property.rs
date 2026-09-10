use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_references_filtering_mapped_type_property() {
    let content = r#"const obj = { /*1*/a: 1, b: 2 };
const filtered: { [P in keyof typeof obj as P extends 'b' ? never : P]: 0; } = { /*2*/a: 0 };
filtered./*3*/a;"#;
    let mut s = Session::new_for_test("findAllReferencesFilteringMappedTypeProperty", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
