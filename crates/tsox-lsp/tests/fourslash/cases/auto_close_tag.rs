use tsox_lsp::fourslash::Session;


#[test]
fn auto_close_tag() {
    // TODO: // Using separate files for each example to avoid unclosed JSX tags affecting other tests.
    let content = r#"// @noLib: true

// @Filename: /0.tsx
const x = <div>/*0*/;

// @Filename: /1.tsx
const x = <div> foo/*1*/ </div>;

// @Filename: /2.tsx
const x = <div></div>/*2*/;

// @Filename: /3.tsx
const x = <div/>/*3*/;

// @Filename: /4.tsx
const x = <div>
    <p>/*4*/
    </div>
</p>;

// @Filename: /5.tsx
const x = <div> text /*5*/;

// @Filename: /6.tsx
const x = <div>
    <div>/*6*/
</div>;

// @Filename: /7.tsx
const x = <div>
    <p>/*7*/
</div>;

// @Filename: /8.tsx
const x = <div>
    <div>/*8*/</div>
</div>;

// @Filename: /9.tsx
const x = <p>
    <div>
        <div>/*9*/
    </div>
</p>"#;
    let _s = Session::new_for_test("autoCloseTag", content);
    // TODO: f.VerifyJsxClosingTag(t, map[string]*string{
}
