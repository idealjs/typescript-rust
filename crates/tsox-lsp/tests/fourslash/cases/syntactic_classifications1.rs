use tsox_lsp::fourslash::Session;


#[test]
fn syntactic_classifications1() {
    let content = r#"// comment
namespace M {
    var v = 0 + 1;
    var s = "string";

    class C<T> {
    }

    enum E {
    }

    interface I {
    }

    namespace M1.M2 {
    }
}"#;
    let _s = Session::new_for_test("syntacticClassifications1", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
