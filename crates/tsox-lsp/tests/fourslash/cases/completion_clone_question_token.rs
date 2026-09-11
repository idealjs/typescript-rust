use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_clone_question_token() {
    let content = r#"// @strict: false
// @Filename: /file2.ts
type TCallback<T = any> = (options: T) => any;
type InKeyOf<E> = { [K in keyof E]?: TCallback<E[K]>; };
export class Bar<A> {
    baz(a: InKeyOf<A>): void { }
}
// @Filename: /file1.ts
import { Bar } from './file2';
type TwoKeys = Record<'a' | 'b', { thisFails?: any; }>
class Foo extends Bar<TwoKeys> {
    /**/
}"#;
    let mut s = Session::new_for_test("completionCloneQuestionToken", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
