function log(target: any, key: string, desc: any) { return desc; }
export class Greeter {
    greeting: string;
    constructor(msg: string) { this.greeting = msg; }
    @log greet() { return "Hello, " + this.greeting; }
}
