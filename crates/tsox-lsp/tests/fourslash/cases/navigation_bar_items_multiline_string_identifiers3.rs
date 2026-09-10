use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_multiline_string_identifiers3() {
    let content = r#"declare module 'MoreThanOneHundredAndFiftyCharacters\
MoreThanOneHundredAndFiftyCharacters\
MoreThanOneHundredAndFiftyCharacters\
MoreThanOneHundredAndFiftyCharacters\
MoreThanOneHundredAndFiftyCharacters\
MoreThanOneHundredAndFiftyCharacters\
MoreThanOneHundredAndFiftyCharacters' { }"#;
    let mut s = Session::new_for_test("navigationBarItemsMultilineStringIdentifiers3", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
