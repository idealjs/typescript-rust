use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn tsx_signature_help2() {
    let content = r#"// @jsx: preserve
//@Filename: file.tsx
import React = require('react');
export interface ClickableProps {
    children?: string;
    className?: string;
}
export interface ButtonProps extends ClickableProps {
    onClick(event?: React.MouseEvent<HTMLButtonElement>): void;
}
export interface LinkProps extends ClickableProps {
    goTo(where: "home" | "contact"): void;
}
function _buildMainButton({ onClick, children, className }: ButtonProps): JSX.Element {
    return(<button className={className} onClick={onClick}>{ children || 'MAIN BUTTON'}</button>);
}
export function MainButton(buttonProps: ButtonProps): JSX.Element;
export function MainButton(linkProps: LinkProps): JSX.Element;
export function MainButton(props: ButtonProps | LinkProps): JSX.Element {
    return this._buildMainButton(props);
}
let e1 = <MainButton/*1*/ /*2*/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "MainButton(buttonProps: ButtonP
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "MainButton(buttonProps: ButtonP
}
