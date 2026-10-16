use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_namespace_name() {
    let content = r#"{ namespace /*0*/ }
namespace N/*1*/ {}
namespace N.M {}
namespace N./*2*/

namespace N1.M/*3*/ {}
namespace N2.M {}
namespace N2.M/*4*/"#;
    let mut s = Session::new_for_test("completionsNamespaceName", content);
    // TODO: f.VerifyCompletions(t, []string{"0", "1"}, &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["M"]);
    fourslash::verify_completions_empty_at(&mut s, Some("3"));
    fourslash::verify_completions_exact_at(&mut s, Some("4"), &["M"]);
}
