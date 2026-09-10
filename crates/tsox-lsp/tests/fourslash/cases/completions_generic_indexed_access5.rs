use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_generic_indexed_access5() {
    let content = r#"interface CustomElements {
  'component-one': {
      foo?: string;
  },
  'component-two': {
      bar?: string;
  }
}

interface Options<T extends keyof CustomElements> {
    props?: {} & { x: CustomElements[(T extends string ? T : never) & string][] }['x'];
}

declare function f<T extends keyof CustomElements>(k: T, options: Options<T>): void;

f("component-one", {
    props: [{
        /**/
    }]
})"#;
    let mut s = Session::new_for_test("completionsGenericIndexedAccess5", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
