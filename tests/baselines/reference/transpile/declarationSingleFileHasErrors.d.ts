//// [declarationSingleFileHasErrors.ts] ////
export const a number = "missing colon";
//// [declarationSingleFileHasErrors.d.ts] ////
export declare const a: any, number = "missing colon";


//// [Diagnostics reported]
declarationSingleFileHasErrors.ts(1,14): error TS9010: Variable must have an explicit type annotation with --isolatedDeclarations.