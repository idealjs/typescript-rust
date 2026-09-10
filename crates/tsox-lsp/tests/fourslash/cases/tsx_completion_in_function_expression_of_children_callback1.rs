use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_completion_in_function_expression_of_children_callback1() {
    let content = r#"//@module: commonjs
//@jsx: preserve
// @Filename: 1.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props; }
    interface ElementChildrenAttribute { children; }
}
interface IUser {
    Name: string;
}
interface IFetchUserProps {
    children: (user: IUser) => any;
}
function FetchUser(props: IFetchUserProps) { return undefined; }
function UserName() {
    return (
        <FetchUser>
            { user => (
                <h1>{ user./**/ }</h1>
            )}
        </FetchUser>
    );
}"#;
    let mut s = Session::new_for_test("tsxCompletionInFunctionExpressionOfChildrenCallback1", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["Name"]);
}
