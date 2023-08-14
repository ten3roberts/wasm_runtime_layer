# wasm_runtime_layer

[![Crates.io](https://img.shields.io/crates/v/wasm_runtime_layer.svg)](https://crates.io/crates/wasm_runtime_layer)
[![Docs.rs](https://docs.rs/wasm_runtime_layer/badge.svg)](https://docs.rs/wasm_runtime_layer)

`wasm_runtime_layer` creates a thin abstraction over WebAssembly runtimes, allowing for backend-agnostic host code. The interface is based upon the `wasmtime` and `wasmi` crates, but may be implemented for any runtime.

## Usage

To use this crate, first instantiate a backend runtime. The runtime may be any
value that implements `backend::WasmEngine`. Some runtimes are already implemented as optional features.
Then, one can create an `Engine` from the backend runtime, and use it to initialize modules and instances:

```rust
// 1. Instantiate a runtime
let engine = Engine::new(wasmi::Engine::default());
let mut store = Store::new(&engine, ());

// 2. Create modules and instances, similar to other runtimes
let module_bin = wabt::wat2wasm(
    r#"
(module
(type $t0 (func (param i32) (result i32)))
(func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
    get_local $p0
    i32.const 1
    i32.add))
"#,
)
.unwrap();

let module = Module::new(&engine, std::io::Cursor::new(&module_bin)).unwrap();
let instance = Instance::new(&mut store, &module, &Imports::default()).unwrap();

let add_one = instance
    .get_export(&store, "add_one")
    .unwrap()
    .into_func()
    .unwrap();
        
let mut result = [Value::I32(0)];
add_one
    .call(&mut store, &[Value::I32(42)], &mut result)
    .unwrap();

assert_eq!(result[0], Value::I32(43));
```

## Optional features and backends

**backend_wasmi** - Implements the `WasmEngine` trait for `wasmi::Engine` instances.

Contributions for additional backend implementations are welcome!