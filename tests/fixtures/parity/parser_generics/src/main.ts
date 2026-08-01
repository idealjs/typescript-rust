// Generic type parameters and inference
function identity<T>(value: T): T {
  return value;
}

function pair<A, B>(a: A, b: B): [A, B] {
  return [a, b];
}

function merge<T extends object, U extends object>(a: T, b: U): T & U {
  return { ...a, ...b };
}

// Generic constraints
function getProperty<T, K extends keyof T>(obj: T, key: K): T[K] {
  return obj[key];
}

// Generic class
class Container<T> {
  items: T[] = [];
  add(item: T): void {}
  get(index: number): T { return this.items[index]; }
}

// Generic interface
interface Repository<T> {
  findById(id: string): T | null;
  findAll(): T[];
  save(entity: T): T;
}

// Conditional types with generics
type IsString<T> = T extends string ? true : false;
type ElementOf<T> = T extends (infer E)[] ? E : never;
