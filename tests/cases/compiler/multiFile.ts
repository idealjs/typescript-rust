// @module: commonjs
// @target: esnext

// @filename: a.ts
export const greet = (name: string): string => `Hello, ${name}!`;

// @filename: b.ts
import { greet } from "./a";
console.log(greet("world"));
