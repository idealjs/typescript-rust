use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_generic_indexed_access4() {
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

declare function create<T extends 'hello' | 'goodbye'>(name: T, options: Options<T extends 'hello' ? 'component-one' : 'component-two'>): void;
declare function create<T extends keyof CustomElements>(name: T, options: Options<T>): void;

create('hello', { props: { /*1*/ } })
create('goodbye', { props: { /*2*/ } })
create('component-one', { props: { /*3*/ } });"#;
    let mut s = Session::new_for_test("completionsGenericIndexedAccess4", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
