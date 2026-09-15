use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn get_jsx_outlining_spans() {
    let content = r#"import React, { Component } from 'react';

export class Home extends Component[| {
  render()[| {
    return [|(
    [|<div>
      [|<h1>Hello, world!</h1>|]
      [|<ul>
        [|<li>
          [|<a [|href='https://get.asp.net/'|]>
            ASP.NET Core
          </a>|]
        </li>|]
        [|<li>[|<a [|href='https://facebook.github.io/react/'|]>React</a>|] for client-side code</li>|]
        [|<li>[|<a [|href='http://getbootstrap.com/'|]>Bootstrap</a>|] for layout and styling</li>|]
      </ul>|]
      <div
        [|accesskey="test"
        class="active"
        dir="auto"|] />
      <PageHeader [|title="Log in"
        {...[|{
          item: true,
          xs: 9,
          md: 5
        }|]}|]
      />
      [|<>
          text 
      </>|]
    </div>|]
    )|];
  }|]
}|]"#;
    let _s = Session::new_for_test("getJSXOutliningSpans", content);
    // TODO: f.VerifyOutliningSpans(t)
}
