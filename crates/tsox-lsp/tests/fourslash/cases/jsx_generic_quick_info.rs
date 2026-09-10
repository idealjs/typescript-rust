use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("jsxGenericQuickInfo", content);
    fourslash::verify_quick_info_at(&mut s, "0", "(property) PropsA<number>.renderItem: (item: number) => string", "");
    fourslash::verify_quick_info_at(&mut s, "1", "(parameter) item: number", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) PropsA<T>.name: \"A\"", "comments for A");
    fourslash::verify_quick_info_at(&mut s, "3", "(property) PropsA<number>.renderItem: (item: number) => string", "");
    fourslash::verify_quick_info_at(&mut s, "4", "(parameter) item: number", "");
    fourslash::verify_quick_info_at(&mut s, "5", "(property) PropsA<T>.name: \"A\"", "comments for A");
}
