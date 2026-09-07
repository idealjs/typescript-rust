use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_string_literal_types3() {
    let content = r#"type Foo = "[|a|]" | "b";

class C {
    p: Foo = "[|a|]";
    m() {
        switch (this.p) {
            case "[|a|]":
                return 1;
            case "b":
                return 2;
        }
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "a")
}
