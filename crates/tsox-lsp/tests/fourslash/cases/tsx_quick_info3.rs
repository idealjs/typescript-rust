use tsox_lsp::fourslash::{self, Session};

#[test]
fn tsx_quick_info3() {
    let content = r#"//@Filename: file.tsx
// @jsx: preserve
// @noLib: true
interface OptionProp {
    propx: 2
}
class Opt extends React.Component<OptionProp, {}> {
    render() {
        return <div>Hello</div>;
    }
}
const obj1: OptionProp = {
    propx: 2
}
let y1 = <O/*1*/pt pro/*2*/px={2} />;
let y2 = <Opt {...ob/*3*/j1} />;
let y2 = <Opt {...obj1} pr/*4*/opx />;"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "class Opt", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) propx: number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "const obj1: OptionProp", "");
    fourslash::verify_quick_info_at(&mut s, "4", "(property) propx: true", "");
}
