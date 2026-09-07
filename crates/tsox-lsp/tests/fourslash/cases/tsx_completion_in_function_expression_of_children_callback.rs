use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn tsx_completion_in_function_expression_of_children_callback() {
    let content = r#"//@module: commonjs
//@jsx: preserve
// @Filename: 1.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props; }
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", nil)
}
