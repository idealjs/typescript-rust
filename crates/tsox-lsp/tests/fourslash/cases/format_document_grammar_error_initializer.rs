use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_document_grammar_error_initializer() {
    let content = r#"// @Filename: /a.ts
const f = () => {
  const v = x as A & {
    a: { b: C
  }
  const m: T[] = [
    { g: () => { nav(`${z}`) } },
  ]
  const n: T[] =
}
"#;
    let mut s = Session::new_for_test("formatDocumentGrammarErrorInitializer", content);
    fourslash::format_document(&mut s, "/a.ts");
    // TODO: }
}
