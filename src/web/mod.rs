mod store;
use slab::Slab;
pub use store::{Store, StoreContext, StoreContextMut, StoreInner};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use js_sys::{Array, Function, WebAssembly};

use crate::{
    backend::{AsContext, AsContextMut, Extern, Value, WasmFunc},
    ExternType,
};

/// Handle used to retain the lifetime of Js passed objects and drop them at an appropriate time.
///
/// Most commonly this is to ensure a closure with captures does not get dropped by Rust while a
/// reference to it exists in the world of Js.
#[derive(Debug)]
pub(crate) struct DropResource(Box<dyn std::fmt::Debug>);

impl DropResource {
    pub fn new(value: impl 'static + std::fmt::Debug) -> Self {
        Self(Box::new(value))
    }

    pub fn from_boxed(value: Box<dyn std::fmt::Debug>) -> Self {
        Self(value)
    }
}

#[derive(Default, Debug, Clone)]
/// Runtime for WebAssembly
pub struct Engine {
    inner: Rc<RefCell<EngineInner>>,
}

impl Engine {
    pub(crate) fn borrow(&self) -> Ref<EngineInner> {
        self.inner.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> RefMut<EngineInner> {
        self.inner.borrow_mut()
    }
}

#[derive(Default, Debug)]
pub(crate) struct EngineInner {
    pub(crate) modules: Slab<ModuleInner>,
}

impl EngineInner {
    pub fn insert_module(&mut self, module: ModuleInner, imports: Vec<Import>) -> Module {
        Module {
            id: self.modules.insert(module),
            imports,
        }
    }
}

/// Not Send + Sync
#[derive(Debug)]
pub(crate) struct InstanceInner {
    pub(crate) instance: WebAssembly::Instance,
    pub(crate) exports: HashMap<String, Extern<Engine>>,
}

/// A WebAssembly Instance.
#[derive(Debug, Clone)]
pub struct Instance {
    pub(crate) id: usize,
}

/// Internal represenation of [`Func`]
#[derive(Debug)]
pub(crate) struct FuncInner {
    pub(crate) func: Function,
}

/// A bound function
#[derive(Debug, Clone)]
pub struct Func {
    pub(crate) id: usize,
}

impl WasmFunc<Engine> for Func {
    fn new<T>(
        mut ctx: impl AsContextMut<Engine, UserState = T>,
        ty: crate::FuncType,
        func: impl 'static
            + Send
            + Sync
            + Fn(StoreContextMut<T>, &[Value<Engine>], &mut [Value<Engine>]) -> anyhow::Result<()>,
    ) -> Self {
        let _span = tracing::info_span!("Func::new").entered();

        let mut ctx: StoreContextMut<_> = ctx.as_context_mut();

        let store = ctx.store();
        let closure: Closure<dyn Fn(Array) -> JsValue> = Closure::new(move |args: Array| {
            tracing::info!(?args, "called");

            JsValue::UNDEFINED
        });

        let func = ctx.insert_func(FuncInner {
            func: closure.as_ref().unchecked_ref::<Function>().clone(),
        });

        tracing::debug!(id = func.id, "func");
        ctx.insert_drop_resource(DropResource::new(closure));

        func
    }

    fn ty(&self, ctx: impl AsContext<Engine>) -> crate::FuncType {
        todo!()
    }

    fn call<T>(
        &self,
        ctx: impl AsContextMut<Engine>,
        args: &[Value<Engine>],
        results: &mut [Value<Engine>],
    ) -> anyhow::Result<()> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Memory {}

#[derive(Debug)]
pub(crate) struct ModuleInner {
    pub(crate) module: js_sys::WebAssembly::Module,
}

#[derive(Debug, Clone)]
pub(crate) struct Import {
    pub(crate) module: String,
    pub(crate) name: String,
    pub(crate) kind: ExternType,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub(crate) id: usize,
    pub(crate) imports: Vec<Import>,
}

/// A global variable accesible as an import or export in a module
///
/// Stored within the store
#[derive(Debug, Clone)]
pub struct Global {
    pub(crate) id: usize,
}

#[derive(Debug)]
pub(crate) struct GlobalInner {
    pub(crate) value: WebAssembly::Global,
}
