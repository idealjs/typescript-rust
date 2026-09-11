use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_string_literal_types1() {
    let content = r#"interface AnimationOptions {
    deltaX: number;
    deltaY: number;
    easing: "ease-in" | "ease-out" | "[|ease-in-out|]";
}

function animate(o: AnimationOptions) { }

animate({ deltaX: 100, deltaY: 100, easing: "[|ease-in-out|]" });"#;
    let mut s = Session::new_for_test("renameStringLiteralTypes1", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "ease-in-out")
}
