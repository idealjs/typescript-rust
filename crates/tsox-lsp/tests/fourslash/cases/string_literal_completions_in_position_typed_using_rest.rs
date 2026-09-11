use tsox_lsp::fourslash::{self, Session};


#[test]
fn string_literal_completions_in_position_typed_using_rest() {
    let content = r#"declare function pick<T extends object, K extends keyof T>(obj: T, ...keys: K[]): Pick<T, K>;
declare function pick2<T extends object, K extends (keyof T)[]>(obj: T, ...keys: K): Pick<T, K[number]>;

const obj = { aaa: 1, bbb: '2', ccc: true };

pick(obj, 'aaa', '/*ts1*/');
pick2(obj, 'aaa', '/*ts2*/');
class Q<T> {
  public select<Keys extends keyof T>(...args: Keys[]) {}
}
new Q<{ id: string; name: string }>().select("name", "/*ts3*/");"#;
    let mut s = Session::new_for_test("stringLiteralCompletionsInPositionTypedUsingRest", content);
    // TODO: f.VerifyCompletions(t, []string{"ts1", "ts2"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"ts3"}, &fourslash.CompletionsExpectedList{
}
