use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("navigationBarItemsComputedNames", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
