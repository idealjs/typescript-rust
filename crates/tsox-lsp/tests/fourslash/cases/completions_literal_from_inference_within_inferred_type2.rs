use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_literal_from_inference_within_inferred_type2() {
    let content = r#"// @Filename: /a.tsx
type Values<T> = T[keyof T];

type GetStates<T> = T extends { states: object } ? T["states"] : never;

type IsNever<T> = [T] extends [never] ? 1 : 0;

type GetIds<T, Gathered extends string = never> = IsNever<T> extends 1
  ? Gathered
  : "id" extends keyof T
  ? GetIds<Values<GetStates<T>>, Gathered | ` + "`" + `#${T["id"] & string}` + "`" + `>
  : GetIds<Values<GetStates<T>>, Gathered>;

type StateConfig<
  TStates extends Record<string, StateConfig> = Record<
    string,
    StateConfig<any>
  >,
  TIds extends string = string
> = {
  id?: string;
  initial?: keyof TStates & string;
  states?: {
    [K in keyof TStates]: StateConfig<GetStates<TStates[K]>, TIds>;
  };
  on?: Record<string, TIds | ` + "`" + `.${keyof TStates & string}` + "`" + `>;
};

declare function createMachine<const T extends StateConfig<GetStates<T>, GetIds<T>>>(
  config: T
): void;

createMachine({
  initial: "child",
  states: {
    child: {
      initial: "foo",
      states: {
        foo: {
          id: "wow_deep_id",
        },
      },
    },
  },
  on: {
    EV: "/*ts*/",
  },
});"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
}
