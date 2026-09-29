use tsox_lsp::fourslash::Session;


#[test]
fn navbar01() {
    let content = r#"// @lib: es5
// Interface
interface IPoint {
    getDist(): number;
    new(): IPoint;
    (): any;
    [x:string]: number;
    prop: string;
}

/// Module
namespace Shapes {
    // Class
    export class Point implements IPoint {
        constructor (public x: number, public y: number) { }

        // Instance member
        getDist() { return Math.sqrt(this.x * this.x + this.y * this.y); }

        // Getter
        get value(): number { return 0; }

        // Setter
        set value(newValue: number) { return; }

        // Static member
        static origin = new Point(0, 0);

        // Static method
        private static getOrigin() { return Point.origin;}
    }

    enum Values { value1, value2, value3 }
}

// Local variables
var p: IPoint = new Shapes.Point(3, 4);
var dist = p.getDist();"#;
    let _s = Session::new_for_test("navbar01", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
