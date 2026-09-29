use tsox_lsp::fourslash::{self, Session};


#[test]
fn property_duplicate_identifier_error() {
    let content = r#"export class C {
    x: number;
    get x(): number { return 1; }
}/*1*/"#;
    let mut s = Session::new_for_test("propertyDuplicateIdentifierError", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "/n");
}
