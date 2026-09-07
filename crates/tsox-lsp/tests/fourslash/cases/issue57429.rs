use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNonSuggestionDiagnostics"]
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
    return ` + "`" + `${value}` + "`" + `
  },
  doAnotherThing() { },
})"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "const value: any", "");
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
