use tsox_lsp::fourslash::{self, Session};


#[test]
fn issue57429() {
    let content = r#"// @strict: true
function Builder<I>(def: I) {
  return def;
}

interface IThing {
  doThing: (args: { value: object }) => string
  doAnotherThing: () => void
}

Builder<IThing>({
  doThing(args: { value: object }) {
    const { v/*1*/alue } = this.[|args|]
    return `${value}`
  },
  doAnotherThing() { },
})"#;
    let mut s = Session::new_for_test("issue57429", content);
    fourslash::verify_quick_info_at(&mut s, "1", "const value: any", "");
    // TODO: f.VerifyNonSuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
