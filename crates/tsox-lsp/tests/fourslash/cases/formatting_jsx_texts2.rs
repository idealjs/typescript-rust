use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_jsx_texts2() {
    let content = r#"//@Filename: file.tsx
const a = (
    <div>
  foo
          </div>
);

const b = (
    <div>
  {     foo  }
          </div>
);

const c = (
    <div>
    foo
  {     foobar  }
  bar
          </div>
);

const d = 
    <div>
  foo
          </div>;

const e = 
    <div>
  {     foo  }
          </div>

const f = 
    <div>
    foo
  {     foobar  }
  bar
          </div>"#;
    let mut s = Session::new_for_test("formattingJsxTexts2", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"const a = (
    <div>
        foo
    </div>
);

const b = (
    <div>
        {foo}
    </div>
);

const c = (
    <div>
        foo
        {foobar}
        bar
    </div>
);

const d =
    <div>
        foo
    </div>;

const e =
    <div>
        {foo}
    </div>

const f =
    <div>
        foo
        {foobar}
        bar
    </div>"#);
}
