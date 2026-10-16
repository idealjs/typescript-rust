use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_chaining_methods() {
    let content = r#" z$ = this.store.select(this.fake())
     .ofType(
      'ACTION',
      'ACTION-2'
     )
     .pipe(
         filter(x => !!x),
         switchMap(() =>
          this.store.select(this.menuSelector.getAll('x'))
           .pipe(
             tap(x => {
             this.x = !x;
             })
           )
         )
     );

1
    .toFixed(
        2);"#;
    let mut s = Session::new_for_test("formattingChainingMethods", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"z$ = this.store.select(this.fake())
    .ofType(
        'ACTION',
        'ACTION-2'
    )
    .pipe(
        filter(x => !!x),
        switchMap(() =>
            this.store.select(this.menuSelector.getAll('x'))
                .pipe(
                    tap(x => {
                        this.x = !x;
                    })
                )
        )
    );

1
    .toFixed(
        2);"#);
}
