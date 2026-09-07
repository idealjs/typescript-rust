use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
