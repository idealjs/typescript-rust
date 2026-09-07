use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn tsx_signature_help1() {
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
function _buildMainButton({ onClick, children, className }: ButtonProps): JSX.Element {
    return(<button className={className} onClick={onClick}>{ children || 'MAIN BUTTON'}</button>);
}
export function MainButton(props: ButtonProps): JSX.Element {
    return this._buildMainButton(props);
}
let e1 = <MainButton/*1*/ /*2*/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "MainButton(props: ButtonProps):
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "MainButton(props: ButtonProps):
}
