use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_multiline_string_identifiers1() {
    let content = r#"declare module "Multiline\r\nMadness" {
}

declare module "Multiline\
Madness" {
}
declare module "MultilineMadness" {}

declare module "Multiline\
Madness2" {
}

interface Foo {
    "a1\\\r\nb";
    "a2\
    \
    b"(): Foo;
}

class Bar implements Foo {
    'a1\\\r\nb': Foo;

    'a2\
    \
    b'(): Foo {
        return this;
    }
}"#;
    let mut s = Session::new_for_test("navigationBarItemsMultilineStringIdentifiers1", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
