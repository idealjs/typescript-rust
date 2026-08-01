// Conditional types, mapped types, and utility types
type NonNullable<T> = T extends null | undefined ? never : T;
type Awaited<T> = T extends Promise<infer U> ? U : T;
type Unpack<T> = T extends Array<infer U> ? U : T extends Promise<infer U> ? U : T;

// Mapped types
type Readonly<T> = { readonly [P in keyof T]: T[P]; };
type Partial<T> = { [P in keyof T]?: T[P]; };
type Pick<T, K extends keyof T> = { [P in K]: T[P]; };
type Record<K extends keyof any, T> = { [P in K]: T; };

// Template literal types
type Hello = `hello ${string}`;
type Email = `${string}@${string}.${string}`;

// Indexed access types
type ElementOf<T> = T extends (infer E)[] ? E : never;
type Props = { a: number; b: string; c: boolean };
type A = Props["a"];
type AB = Props["a" | "b"];

// Keyof and typeof
const config = { port: 3000, host: "localhost" };
type Config = typeof config;
type ConfigKeys = keyof Config;

// Function type with overloads
type StringOrNumber = string | number;
type Mapper<T> = (item: T) => StringOrNumber;
