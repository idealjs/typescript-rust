use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn tsx_find_all_references11() {
    let content = r#"//@Filename: file.tsx
// @jsx: preserve
// @noLib: true
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props; }
}
interface ClickableProps {
    children?: string;
    className?: string;
}
interface ButtonProps extends ClickableProps {
    onClick(event?: React.MouseEvent<HTMLButtonElement>): void;
}
interface LinkProps extends ClickableProps {
    goTo: string;
}
declare function MainButton(buttonProps: ButtonProps): JSX.Element;
declare function MainButton(linkProps: LinkProps): JSX.Element;
declare function MainButton(props: ButtonProps | LinkProps): JSX.Element;
let opt = <MainButton /*1*/wrong />;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
