use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn jsx_generic_quick_info() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props }
}
interface PropsA<T> {
    /** comments for A */
    name: 'A',
    items: T[];
    renderItem: (item: T) => string;
}
interface PropsB<T> {
    /** comments for B */
    name: 'B',
    items: T[];
    renderItem: (item: T) => string;
}
class Component<T> {
    constructor(props: PropsA<T> | PropsB<T>) {}
    props: PropsA<T> | PropsB<T>;
}   
var b = new Component({items: [0, 1, 2], render/*0*/Item: it/*1*/em => item.toFixed(), name/*2*/: 'A',});
var c = <Component items={[0, 1, 2]} render/*3*/Item={it/*4*/em => item.toFixed()} name/*5*/="A" />"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "0", "(property) PropsA<number>.renderItem: (item: number) => string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(parameter) item: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(property) PropsA<T>.name: \"A\"", "comments for A")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(property) PropsA<number>.renderItem: (item: number) => string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(parameter) item: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(property) PropsA<T>.name: \"A\"", "comments for A")
}
