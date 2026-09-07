use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_computed_names() {
    let content = r#"const enum E {
	A = 'A',
}
const a = '';

class C {
    [a]() {
        return 1;
    }

    [E.A]() {
        return 1;
    }

    [1]() {
        return 1;
    },

    ["foo"]() {
        return 1;
    },
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
