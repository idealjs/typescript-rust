use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_multiline_string_identifiers2() {
    let content = r#"function f(p1: () => any, p2: string) { }
f(() => { }, `line1\
line2\
line3`);

class c1 {
    const a = ' ''line1\
        line2';
}

f(() => { }, `unterminated backtick 1
unterminated backtick 2
unterminated backtick 3"#;
    let mut s = Session::new_for_test("navigationBarItemsMultilineStringIdentifiers2", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
