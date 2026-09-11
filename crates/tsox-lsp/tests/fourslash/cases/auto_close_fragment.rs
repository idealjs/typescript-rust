use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_close_fragment() {
    // TODO: // Using separate files for each example to avoid unclosed JSX tags affecting other tests.
    let content = r#"// @noLib: true
// @Filename: /0.tsx
const x = <>/*0*/;

// @Filename: /1.tsx
const x = <> foo/*1*/ </>;

// @Filename: /2.tsx
const x = <></>/*2*/;

// @Filename: /3.tsx
const x = </>/*3*/;

// @Filename: /4.tsx
const x = <div>
    <>/*4*/
    </div>
</>;

// @Filename: /5.tsx
const x = <> text /*5*/;

// @Filename: /6.tsx
const x = <>
    <>/*6*/
</>;

// @Filename: /7.tsx
const x = <div>
    <>/*7*/
</div>;

// @Filename: /8.tsx
const x = <div>
    <>/*8*/</>
</div>;

// @Filename: /9.tsx
const x = <p>
    <>
        <>/*9*/
    </>
</p>"#;
    let mut s = Session::new_for_test("autoCloseFragment", content);
    // TODO: f.VerifyJsxClosingTag(t, map[string]*string{
}
