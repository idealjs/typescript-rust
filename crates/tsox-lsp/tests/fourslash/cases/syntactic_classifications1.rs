use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
