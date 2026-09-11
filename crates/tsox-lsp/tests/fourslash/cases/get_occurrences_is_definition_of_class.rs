use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_is_definition_of_class() {
    let content = r#"/*1*/class /*2*/C {
    n: number;
    constructor() {
        this.n = 12;
    }
}
let c = new /*3*/C();"#;
    let mut s = Session::new_for_test("getOccurrencesIsDefinitionOfClass", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
