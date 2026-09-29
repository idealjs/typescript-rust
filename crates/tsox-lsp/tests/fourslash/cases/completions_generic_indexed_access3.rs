use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_generic_indexed_access3() {
    let content = r#"interface CustomElements {
  'component-one': {
      foo?: string;
  },
  'component-two': {
      bar?: string;
  }
}

interface Options<T extends keyof CustomElements> {
  props: CustomElements[T];
}

declare function create<T extends keyof CustomElements>(name: T, options: Options<T>): void;

create('component-one', { props: { /*1*/ } });
create('component-two', { props: { /*2*/ } });"#;
    let mut s = Session::new_for_test("completionsGenericIndexedAccess3", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
