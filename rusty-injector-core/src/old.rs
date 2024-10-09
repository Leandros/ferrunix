#![allow(
    missing_docs,
    missing_debug_implementations,
    clippy::missing_docs_in_private_items,
    clippy::missing_panics_doc
)]
//! Rust Runtime Dependency Injection Framework.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

pub mod test;

enum Object {
    Singleton(SingletonObject),
    Transient(TransientObject),
}

struct SingletonObject {
    pub deps: Option<Vec<TypeId>>,
    pub obj: Arc<dyn std::any::Any + Send + Sync>,
}

struct TransientObject {
    pub deps: Option<Vec<TypeId>>,
    pub ctor: Box<
        dyn Fn(&Registry, &Option<Vec<TypeId>>) -> Box<dyn std::any::Any + Send + Sync>
            + Send
            + Sync,
    >,
}

#[non_exhaustive]
pub enum Dependency {
    Empty,
    Singleton(Arc<dyn std::any::Any + Send + Sync>),
    Transient(Box<dyn std::any::Any + Send + Sync>),
}

impl Dependency {
    pub fn transient<T: 'static>(self) -> T {
        if let Self::Transient(transient) = self {
            return *transient.downcast::<T>().expect("bug");
        }

        unreachable!("bug")
    }

    pub fn singleton<T: Send + Sync + 'static>(&self) -> Arc<T> {
        if let Self::Singleton(singleton) = self {
            return Arc::clone(singleton).downcast::<T>().expect("bug");
        }

        unreachable!("bug")
    }
}

pub struct Registry {
    /// Singletons live for the entire registry live time.
    objects: HashMap<std::any::TypeId, Object>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    pub fn transient<T>(&mut self, ctor: fn() -> T)
    where
        T: Send + Sync + 'static,
    {
        self.objects.insert(
            TypeId::of::<T>(),
            Object::Transient(TransientObject {
                deps: None,
                ctor: Box::new(move |_, _| -> Box<dyn std::any::Any + Send + Sync> {
                    let obj = ctor();
                    Box::new(obj)
                }),
            }),
        );
    }

    pub fn transient_deps<T, D>(&mut self, ctor: fn(Vec<Dependency>) -> T, deps: D)
    where
        T: Send + Sync + 'static,
        D: Dependable,
    {
        self.objects.insert(
            TypeId::of::<T>(),
            Object::Transient(TransientObject {
                deps: Some(make_deps(deps)),
                ctor: Box::new(move |this, deps| -> Box<dyn std::any::Any + Send + Sync> {
                    let deps = deps.as_ref().expect("this is a bug");
                    let mut obj_deps = Vec::<Dependency>::with_capacity(deps.len());
                    for dep in deps {
                        if let Some(transient) = this.get_transient_dyn(dep) {
                            obj_deps.push(Dependency::Transient(transient));
                        } else if let Some(singleton) = this.get_singleton_dyn(dep) {
                            obj_deps.push(Dependency::Singleton(singleton));
                        }
                    }
                    let obj = ctor(obj_deps);
                    Box::new(obj)
                }),
            }),
        );
    }

    pub fn singleton<T: Send + Sync + 'static>(&mut self, singleton: T) {
        self.objects.insert(
            TypeId::of::<T>(),
            Object::Singleton(SingletonObject {
                deps: None,
                obj: Arc::new(singleton),
            }),
        );
    }

    pub fn get_transient<T: Send + Sync + 'static>(&self) -> Option<T> {
        if let Some(Object::Transient(transient)) = self.objects.get(&TypeId::of::<T>()) {
            let boxed = (transient.ctor)(self, &transient.deps);
            if let Ok(obj) = boxed.downcast::<T>() {
                return Some(*obj);
            }
        }

        None
    }

    pub fn get_transient_dyn(
        &self,
        typeid: &TypeId,
    ) -> Option<Box<dyn std::any::Any + Send + Sync>> {
        if let Some(Object::Transient(transient)) = self.objects.get(typeid) {
            let boxed = (transient.ctor)(self, &transient.deps);
            return Some(boxed);
        }

        None
    }

    pub fn get_singleton<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        if let Some(Object::Singleton(singleton)) = self.objects.get(&TypeId::of::<T>()) {
            if let Ok(typed_obj) = Arc::clone(&singleton.obj).downcast::<T>() {
                return Some(typed_obj);
            }
        }

        None
    }

    pub fn get_singleton_dyn(
        &self,
        typeid: &TypeId,
    ) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        if let Some(Object::Singleton(singleton)) = self.objects.get(typeid) {
            return Some(Arc::clone(&singleton.obj));
        }

        None
    }
}

pub trait Dependable {
    fn to_typeid(self, vec: &mut Vec<TypeId>);
}

#[derive(Debug)]
pub struct Root;

impl Root {
    pub fn and<T: Send + Sync + 'static>(self) -> Dep<Self> {
        Dep {
            head: self,
            args: TypeId::of::<T>(),
        }
    }
}

impl Dependable for Root {
    fn to_typeid(self, _vec: &mut Vec<TypeId>) {
        // nothing to do
    }
}

#[derive(Debug)]
pub struct Dep<H> {
    head: H,
    args: TypeId,
}

impl<H> Dep<H> {
    pub fn and<T: Send + Sync + 'static>(self) -> Dep<Self> {
        Dep {
            head: self,
            args: TypeId::of::<T>(),
        }
    }
}

pub fn make_deps<T: Dependable>(chain: T) -> Vec<TypeId> {
    let mut ret = Vec::new();

    chain.to_typeid(&mut ret);
    ret
}

impl<H: Dependable> Dependable for Dep<H> {
    fn to_typeid(self, vec: &mut Vec<TypeId>) {
        self.head.to_typeid(vec);
        vec.push(self.args);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, PartialEq)]
    struct TestObj {
        pub str: String,
        pub real: f64,
    }

    #[test]
    fn poc() {
        let mut registry = Registry::new();
        registry.singleton(42_u16);
        registry.transient(|| -> u8 { 42_u8 });

        let obj = registry.get_transient::<u8>();
        assert_eq!(obj, Some(42_u8));

        registry.transient(TestObj::default);

        let obj2 = registry.get_transient::<TestObj>();
        assert_eq!(
            obj2,
            Some(TestObj {
                str: String::default(),
                real: f64::default(),
            })
        );

        let obj3 = registry.get_singleton::<u16>().unwrap();
        assert_eq!(*obj3, 42_u16);
    }

    #[test]
    fn pocdep() {
        let mut registry = Registry::new();
        registry.transient(|| -> u8 { 42_u8 });
        registry.transient_deps(
            |mut deps| -> u16 {
                let i = deps.pop().map(|el| el.transient::<u8>()).expect("bug");
                u16::from(i) * 2
            },
            Root.and::<u8>(),
        );

        let obj = registry.get_transient::<u8>();
        assert_eq!(obj, Some(42_u8));

        let obj2 = registry.get_transient::<u16>();
        assert_eq!(obj2, Some(84_u16));
    }

    #[test]
    fn multiple_deps() {
        let mut registry = Registry::new();
        registry.transient(|| -> u8 { 42_u8 });
        registry.transient_deps(
            |mut deps| -> u16 {
                let i = deps.pop().map(|el| el.transient::<u8>()).expect("bug");
                u16::from(i) + 2
            },
            Root.and::<u8>(),
        );
        registry.transient_deps(
            |mut deps| -> u32 {
                let i16 = deps.pop().map(|el| el.transient::<u16>()).expect("bug");
                let i8 = deps.pop().map(|el| el.transient::<u8>()).expect("bug");
                u32::from(i16) + u32::from(i8) + 2
            },
            Root.and::<u8>().and::<u16>(),
        );

        let obj = registry.get_transient::<u32>();
        assert_eq!(obj, Some(88_u32));
    }

    #[test]
    #[allow(clippy::style)]
    fn typeids() {
        let x = Root.and::<u8>().and::<u16>();
        let deps = make_deps(x);
        assert_eq!(deps.get(0), Some(TypeId::of::<u8>()).as_ref());
        assert_eq!(deps.get(1), Some(TypeId::of::<u16>()).as_ref());
    }
}
