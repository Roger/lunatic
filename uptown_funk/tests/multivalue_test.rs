use uptown_funk::{host_functions, Executor, HostFunctions};
use wasmer::{self, Exportable};
use wasmtime;

use std::fs::read;

enum Memory {
    Wasmer(wasmer::Memory),
    Wasmtime(wasmtime::Memory),
}

struct SimpleExcutor {
    memory: Memory,
}

impl Executor for SimpleExcutor {
    fn wasm_memory(&self) -> &mut [u8] {
        match &self.memory {
            Memory::Wasmer(memory) => unsafe { memory.data_unchecked_mut() },
            Memory::Wasmtime(memory) => unsafe { memory.data_unchecked_mut() },
        }
    }
}

struct Empty {}

#[host_functions(namespace = "env")]
impl Empty {
    fn leet(&self) -> (i32, f32, i64) {
        (1337, 1337.1337, 1337)
    }
}

#[test]
fn wasmtime_mutivalue_test() {
    let store = wasmtime::Store::default();
    let wasm = read("tests/wasm/multivalue.wasm")
        .expect("Wasm file not found. Did you run ./build.sh inside the tests/wasm/ folder?");
    let module = wasmtime::Module::new(store.engine(), wasm).unwrap();
    let mut linker = wasmtime::Linker::new(&store);

    let memory_ty = wasmtime::MemoryType::new(wasmtime::Limits::new(32, None));
    let memory = wasmtime::Memory::new(&store, memory_ty);
    linker.define("env", "memory", memory.clone()).unwrap();

    let empty = Empty {};
    let instance_state = SimpleExcutor {
        memory: Memory::Wasmtime(memory),
    };
    empty.add_to_linker(instance_state, &mut linker);

    let instance = linker.instantiate(&module).unwrap();
    let test = instance.get_func("test").unwrap().get0::<()>().unwrap();
    assert_eq!(test().is_ok(), true);
}

#[test]
fn wasmer_mutivalue_test() {
    let store = wasmer::Store::default();
    let wasm = read("tests/wasm/multivalue.wasm")
        .expect("Wasm file not found. Did you run ./build.sh inside the tests/wasm/ folder?");
    let module = wasmer::Module::new(&store, wasm).unwrap();
    let mut wasmer_linker = uptown_funk::wasmer::WasmerLinker::new();

    let memory_ty = wasmer::MemoryType::new(32, None, false);
    let memory = wasmer::Memory::new(&store, memory_ty).unwrap();
    wasmer_linker.add("env", "memory", memory.to_export());

    let empty = Empty {};
    let instance_state = SimpleExcutor {
        memory: Memory::Wasmer(memory),
    };
    empty.add_to_wasmer_linker(instance_state, &mut wasmer_linker, &store);

    let instance = wasmer::Instance::new(&module, &wasmer_linker).unwrap();
    let test = instance
        .exports
        .get_function("test")
        .unwrap()
        .native::<(), ()>()
        .unwrap();

    assert_eq!(test.call().is_ok(), true);
}
