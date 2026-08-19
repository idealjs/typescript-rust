//// [defines.ts] ////
export class A {
    field = { x: 1 }
}
//// [consumes.ts] ////
import {A} from "./defines.js";
export function create() {
    return new A();
}
//// [exposes.ts] ////
import {create} from "./consumes.js";
export const value = create();
//// [defines.d.ts] ////
export declare class A {
    field: {
        x: number;
    };
}
//// [consumes.d.ts] ////
export declare function create(): any;


//// [Diagnostics reported]
consumes.ts(2,17): error TS9007: Function must have an explicit return type annotation with --isolatedDeclarations.
//// [exposes.d.ts] ////
export declare const value: any;


//// [Diagnostics reported]
exposes.ts(2,14): error TS9010: Variable must have an explicit type annotation with --isolatedDeclarations.