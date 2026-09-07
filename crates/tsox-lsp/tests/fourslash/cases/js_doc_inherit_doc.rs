use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn js_doc_inherit_doc() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: inheritDoc.ts
class Foo {
    /**
     * Foo constructor documentation
     */
    constructor(value: number) {}
    /**
     * Foo#method1 documentation
     */
    static method1() {}
    /**
     * Foo#method2 documentation
     */
    method2() {}
    /**
     * Foo#property1 documentation
     */
    property1: string;
    /**
     * Foo#property3 documentation
     */
    property3 = "instance prop";
}
interface Baz {
    /** Baz#property1 documentation */
    property1: string;
    /**
     * Baz#property2 documentation
     */
    property2: object;
}
class Bar extends Foo implements Baz {
    ctorValue: number;
    /** @inheritDoc */
    constructor(value: number) {
        super(value);
        this.ctorValue = value;
    }
    /** @inheritDoc */
    static method1() {}
    method2() {}
    /** @inheritDoc */
    property1: string;
    /**
     * Bar#property2
     * @inheritDoc
     */
    property2: object;

    static /*6*/property3 = "class prop";
}
const b = new Bar/*1*/(5);
b.method2/*2*/();
Bar.method1/*3*/();
const p1 = b.property1/*4*/;
const p2 = b.property2/*5*/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "constructor Bar(value: number): Bar", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(method) Bar.method2(): void", "Foo#method2 documentation")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(method) Bar.method1(): void", "Foo#method1 documentation")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(property) Bar.property1: string", "Foo#property1 documentation")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(property) Bar.property2: object", "Baz#property2 documentation\nBar#pr
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "(property) Bar.property3: string", "")
}
