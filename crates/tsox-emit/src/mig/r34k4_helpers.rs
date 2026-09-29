use std::sync::{Arc, OnceLock};

use tsox_frontend::format::mig::m4o_2::{EmitHelper, Priority};

pub fn decorate_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:decorate".to_owned(),
        scoped: false,
        text: r####"var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};"####.to_owned(),
        text_callback: None,
        priority: Some(Priority { value: 2 }),
        dependencies: vec![],
        import_name: "__decorate".to_owned(),
    }))
}

pub fn add_disposable_resource_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:addDisposableResource".to_owned(),
        scoped: false,
        text: r####"var __addDisposableResource = (this && this.__addDisposableResource) || function (env, value, async) {
    if (value !== null && value !== void 0) {
        if (typeof value !== "object" && typeof value !== "function") throw new TypeError("Object expected.");
        var dispose, inner;
        if (async) {
            if (!Symbol.asyncDispose) throw new TypeError("Symbol.asyncDispose is not defined.");
            dispose = value[Symbol.asyncDispose];
        }
        if (dispose === void 0) {
            if (!Symbol.dispose) throw new TypeError("Symbol.dispose is not defined.");
            dispose = value[Symbol.dispose];
            if (async) inner = dispose;
        }
        if (typeof dispose !== "function") throw new TypeError("Object not disposable.");
        if (inner) dispose = function() { try { inner.call(this); } catch (e) { return Promise.reject(e); } };
        env.stack.push({ value: value, dispose: dispose, async: async });
    }
    else if (async) {
        env.stack.push({ async: true });
    }
    return value;
};"####.to_owned(),
        text_callback: None,
        priority: None,
        dependencies: vec![],
        import_name: "__addDisposableResource".to_owned(),
    }))
}

pub fn dispose_resources_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:disposeResources".to_owned(),
        scoped: false,
        text: r####"var __disposeResources = (this && this.__disposeResources) || (function (SuppressedError) {
    return function (env) {
        function fail(e) {
            env.error = env.hasError ? new SuppressedError(e, env.error, "An error was suppressed during disposal.") : e;
            env.hasError = true;
        }
        var r, s = 0;
        function next() {
            while (r = env.stack.pop()) {
                try {
                    if (!r.async && s === 1) return s = 0, env.stack.push(r), Promise.resolve().then(next);
                    if (r.dispose) {
                        var result = r.dispose.call(r.value);
                        if (r.async) return s |= 2, Promise.resolve(result).then(next, function(e) { fail(e); return next(); });
                    }
                    else s |= 1;
                }
                catch (e) {
                    fail(e);
                }
            }
            if (s === 1) return env.hasError ? Promise.reject(env.error) : Promise.resolve();
            if (env.hasError) throw env.error;
        }
        return next();
    };
})(typeof SuppressedError === "function" ? SuppressedError : function (error, suppressed, message) {
    var e = new Error(message);
    return e.name = "SuppressedError", e.error = error, e.suppressed = suppressed, e;
});"####.to_owned(),
        text_callback: None,
        priority: None,
        dependencies: vec![],
        import_name: "__disposeResources".to_owned(),
    }))
}

pub fn class_private_field_get_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:classPrivateFieldGet".to_owned(),
        scoped: false,
        text: r####"var __classPrivateFieldGet = (this && this.__classPrivateFieldGet) || function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};"####.to_owned(),
        text_callback: None,
        priority: None,
        dependencies: vec![],
        import_name: "__classPrivateFieldGet".to_owned(),
    }))
}

pub fn class_private_field_set_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:classPrivateFieldSet".to_owned(),
        scoped: false,
        text: r####"var __classPrivateFieldSet = (this && this.__classPrivateFieldSet) || function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};"####.to_owned(),
        text_callback: None,
        priority: None,
        dependencies: vec![],
        import_name: "__classPrivateFieldSet".to_owned(),
    }))
}

pub fn class_private_field_in_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:classPrivateFieldIn".to_owned(),
        scoped: false,
        text: r####"var __classPrivateFieldIn = (this && this.__classPrivateFieldIn) || function(state, receiver) {
    if (receiver === null || (typeof receiver !== "object" && typeof receiver !== "function")) throw new TypeError("Cannot use 'in' operator on non-object");
    return typeof state === "function" ? receiver === state : state.has(receiver);
};"####.to_owned(),
        text_callback: None,
        priority: None,
        dependencies: vec![],
        import_name: "__classPrivateFieldIn".to_owned(),
    }))
}

pub fn await_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:await".to_owned(),
        scoped: false,
        text: r####"var __await = (this && this.__await) || function (v) { return this instanceof __await ? (this.v = v, this) : new __await(v); }"####.to_owned(),
        text_callback: None,
        priority: None,
        dependencies: vec![],
        import_name: "__await".to_owned(),
    }))
}

pub fn async_generator_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:asyncGenerator".to_owned(),
        scoped: false,
        text: r####"var __asyncGenerator = (this && this.__asyncGenerator) || function (thisArg, _arguments, generator) {
    if (!Symbol.asyncIterator) throw new TypeError("Symbol.asyncIterator is not defined.");
    var g = generator.apply(thisArg, _arguments || []), i, q = [];
    return i = Object.create((typeof AsyncIterator === "function" ? AsyncIterator : Object).prototype), verb("next"), verb("throw"), verb("return", awaitReturn), i[Symbol.asyncIterator] = function () { return this; }, i;
    function awaitReturn(f) { return function (v) { return Promise.resolve(v).then(f, reject); }; }
    function verb(n, f) { if (g[n]) { i[n] = function (v) { return new Promise(function (a, b) { q.push([n, v, a, b]) > 1 || resume(n, v); }); }; if (f) i[n] = f(i[n]); } }
    function resume(n, v) { try { step(g[n](v)); } catch (e) { settle(q[0][3], e); } }
    function step(r) { r.value instanceof __await ? Promise.resolve(r.value.v).then(fulfill, reject) : settle(q[0][2], r); }
    function fulfill(value) { resume("next", value); }
    function reject(value) { resume("throw", value); }
    function settle(f, v) { if (f(v), q.shift(), q.length) resume(q[0][0], q[0][1]); }
};"####.to_owned(),
        text_callback: None,
        priority: None,
        dependencies: vec![],
        import_name: "__asyncGenerator".to_owned(),
    }))
}

pub fn async_delegator_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:asyncDelegator".to_owned(),
        scoped: false,
        text: r####"var __asyncDelegator = (this && this.__asyncDelegator) || function (o) {
    var i, p;
    return i = {}, verb("next"), verb("throw", function (e) { throw e; }), verb("return"), i[Symbol.iterator] = function () { return this; }, i;
    function verb(n, f) { i[n] = o[n] ? function (v) { return (p = !p) ? { value: __await(o[n](v)), done: false } : f ? f(v) : v; } : f; }
};"####.to_owned(),
        text_callback: None,
        priority: None,
        dependencies: vec![],
        import_name: "__asyncDelegator".to_owned(),
    }))
}

pub fn async_values_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:asyncValues".to_owned(),
        scoped: false,
        text: r####"var __asyncValues = (this && this.__asyncValues) || function (o) {
    if (!Symbol.asyncIterator) throw new TypeError("Symbol.asyncIterator is not defined.");
    var m = o[Symbol.asyncIterator], i;
    return m ? m.call(o) : (o = typeof __values === "function" ? __values(o) : o[Symbol.iterator](), i = {}, verb("next"), verb("throw"), verb("return"), i[Symbol.asyncIterator] = function () { return this; }, i);
    function verb(n) { i[n] = o[n] && function (v) { return new Promise(function (resolve, reject) { v = o[n](v), settle(resolve, reject, v.done, v.value); }); }; }
    function settle(resolve, reject, d, v) { Promise.resolve(v).then(function(v) { resolve({ value: v, done: d }); }, reject); }
};"####.to_owned(),
        text_callback: None,
        priority: None,
        dependencies: vec![],
        import_name: "__asyncValues".to_owned(),
    }))
}

pub fn awaiter_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:awaiter".to_owned(),
        scoped: false,
        text: r####"var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};"####.to_owned(),
        text_callback: None,
        priority: Some(Priority { value: 5 }),
        dependencies: vec![],
        import_name: "__awaiter".to_owned(),
    }))
}

pub fn es_decorate_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:esDecorate".to_owned(),
        scoped: false,
        text: r####"var __esDecorate = (this && this.__esDecorate) || function (ctor, descriptorIn, decorators, contextIn, initializers, extraInitializers) {
    function accept(f) { if (f !== void 0 && typeof f !== "function") throw new TypeError("Function expected"); return f; }
    var kind = contextIn.kind, key = kind === "getter" ? "get" : kind === "setter" ? "set" : "value";
    var target = !descriptorIn && ctor ? contextIn["static"] ? ctor : ctor.prototype : null;
    var descriptor = descriptorIn || (target ? Object.getOwnPropertyDescriptor(target, contextIn.name) : {});
    var _, done = false;
    for (var i = decorators.length - 1; i >= 0; i--) {
        var context = {};
        for (var p in contextIn) context[p] = p === "access" ? {} : contextIn[p];
        for (var p in contextIn.access) context.access[p] = contextIn.access[p];
        context.addInitializer = function (f) { if (done) throw new TypeError("Cannot add initializers after decoration has completed"); extraInitializers.push(accept(f || null)); };
        var result = (0, decorators[i])(kind === "accessor" ? { get: descriptor.get, set: descriptor.set } : descriptor[key], context);
        if (kind === "accessor") {
            if (result === void 0) continue;
            if (result === null || typeof result !== "object") throw new TypeError("Object expected");
            if (_ = accept(result.get)) descriptor.get = _;
            if (_ = accept(result.set)) descriptor.set = _;
            if (_ = accept(result.init)) initializers.unshift(_);
        }
        else if (_ = accept(result)) {
            if (kind === "field") initializers.unshift(_);
            else descriptor[key] = _;
        }
    }
    if (target) Object.defineProperty(target, contextIn.name, descriptor);
    done = true;
};"####.to_owned(),
        text_callback: None,
        priority: Some(Priority { value: 2 }),
        dependencies: vec![],
        import_name: "__esDecorate".to_owned(),
    }))
}

pub fn export_star_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:export-star".to_owned(),
        scoped: false,
        text: r####"var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};"####.to_owned(),
        text_callback: None,
        priority: Some(Priority { value: 2 }),
        dependencies: vec![],
        import_name: "__exportStar".to_owned(),
    }))
}

pub fn create_binding_helper() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:commonjscreatebinding".to_owned(),
        scoped: false,
        text: r####"var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));"####.to_owned(),
        text_callback: None,
        priority: Some(Priority { value: 1 }),
        dependencies: vec![],
        import_name: "__createBinding".to_owned(),
    }))
}

