use tsox_lsp::fourslash::{self, Session};


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
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
