use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_generic_indexed_access6() {
    let content = r#"// @Filename: component.tsx
interface CustomElements {
  'component-one': {
      foo?: string;
  },
  'component-two': {
      bar?: string;
  }
}

type Options<T extends keyof CustomElements> = { kind: T } & Required<{ x: CustomElements[(T extends string ? T : never) & string] }['x']>;

declare function Component<T extends keyof CustomElements>(props: Options<T>): void;

const c = <Component /**/ kind="component-one" />"#;
    let mut s = Session::new_for_test("completionsGenericIndexedAccess6", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
