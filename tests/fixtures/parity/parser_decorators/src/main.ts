// Decorator syntax
function log(target: any, propertyKey: string, descriptor: PropertyDescriptor) {
  const original = descriptor.value;
  descriptor.value = function(...args: any[]) {
    console.log(`Calling ${propertyKey} with`, args);
    return original.apply(this, args);
  };
}

function configurable(value: boolean) {
  return function(target: any, propertyKey: string, descriptor: PropertyDescriptor) {
    descriptor.configurable = value;
  };
}

class Example {
  @log
  greet(name: string): string {
    return `Hello, ${name}`;
  }

  @configurable(false)
  farewell(name: string): string {
    return `Goodbye, ${name}`;
  }
}
