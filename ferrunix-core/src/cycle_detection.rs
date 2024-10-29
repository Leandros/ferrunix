//! Implementation of a cycle detection algorithm for our dependency resolution algorithm.

use std::any::TypeId;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::dependency_builder::{self, DepBuilder};
use crate::types::{
    HashMap, NonAsyncRwLock, Registerable, RegisterableSingleton, Visitor,
};

/// All possible errors during validation.
#[derive(Debug, Clone, PartialEq, Hash)]
#[non_exhaustive]
pub enum ValidationError {
    /// A cycle between dependencies has been detected.
    Cycle,
    /// Dependencies are missing.
    Missing(Vec<MissingDependencies>),
}

impl std::fmt::Display for ValidationError {
    #[allow(clippy::use_debug)]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cycle => write!(fmt, "cycle detected:"),
            Self::Missing(ref all_missing) => {
                writeln!(fmt, "dependencies missing!")?;

                for missing in all_missing {
                    writeln!(
                        fmt,
                        "dependencies missing for {} ({:?}):",
                        missing.ty.1, missing.ty.0
                    )?;
                    for (type_id, type_name) in &missing.deps {
                        writeln!(fmt, " - {type_name} ({type_id:?})")?;
                    }
                    writeln!(fmt, "\n")?;
                }

                Ok(())
            }
        }
    }
}

impl std::error::Error for ValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// All missing `deps` for type `ty`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MissingDependencies {
    /// This is the type that has missing dependencies.
    pub(crate) ty: (TypeId, &'static str),
    /// These are the missing dependencies of `ty`.
    pub(crate) deps: Vec<(TypeId, &'static str)>,
}

impl MissingDependencies {
    /// Returns a reference to a tuple of the [`std::any::TypeId`] and the type name (as returned
    /// from [`std::any::type_name`], therefore, it's "best effort", and might not be correct or
    /// reproducible).
    ///
    /// This is the type that has missing dependencies.
    pub fn ty(&self) -> &(TypeId, &'static str) {
        &self.ty
    }

    /// Returns a reference to a slice of a description of all dependencies that are missing.
    pub fn missing_dependencies(&self) -> &[(TypeId, &'static str)] {
        &self.deps
    }
}

/// Validation whether all dependencies are registered, and the dependency chain has no cycles.
pub(crate) struct DependencyValidator {
    /// The visitor callbacks. Those are necessary because we only want to register each type once
    /// we have collected them all.
    visitor: NonAsyncRwLock<HashMap<TypeId, Visitor>>,
    /// Dependency graph.
    graph:
        NonAsyncRwLock<petgraph::Graph<&'static str, (), petgraph::Directed>>,
    /// Cache of all previously visited types. To avoid infinite recursion and as an optimization.
    visited: NonAsyncRwLock<HashMap<TypeId, petgraph::graph::NodeIndex>>,
    /// All missing dependencies.
    missing: NonAsyncRwLock<HashMap<TypeId, MissingDependencies>>,
    /// Whether we have already visited all visitors.
    visitor_visited: AtomicBool,
    /// Cached validation result.
    validation_cache: NonAsyncRwLock<
        Option<
            Result<
                Vec<petgraph::graph::NodeIndex>,
                petgraph::algo::Cycle<petgraph::graph::NodeIndex>,
            >,
        >,
    >,
}

impl DependencyValidator {
    /// Create a new dependency validator.
    pub(crate) fn new() -> Self {
        Self {
            visitor: NonAsyncRwLock::new(HashMap::new()),
            graph: NonAsyncRwLock::new(petgraph::Graph::new()),
            visited: NonAsyncRwLock::new(HashMap::new()),
            missing: NonAsyncRwLock::new(HashMap::new()),
            visitor_visited: AtomicBool::new(false),
            validation_cache: NonAsyncRwLock::new(None),
        }
    }

    /// Register a new transient, without any dependencies.
    pub(crate) fn add_transient_no_deps<T>(&self)
    where
        T: Registerable,
    {
        let visitor: Visitor = |this| {
            if let Some(index) = this.visited.read().get(&TypeId::of::<T>()) {
                return *index;
            }

            let index = {
                let mut graph = this.graph.write();
                graph.add_node(std::any::type_name::<T>())
            };

            {
                let mut visited = this.visited.write();
                visited.insert(TypeId::of::<T>(), index);
            }

            index
        };

        // TODO: Make this lock free!
        {
            let _ = self.validation_cache.write().take();
        }
        self.visitor_visited.store(false, Ordering::Release);
        self.visitor.write().insert(TypeId::of::<T>(), visitor);
    }

    /// Register a new singleton, without any dependencies.
    pub(crate) fn add_singleton_no_deps<T>(&self)
    where
        T: RegisterableSingleton,
    {
        self.add_transient_no_deps::<T>();
    }

    /// Register a new transient, with dependencies specified via `Deps`.
    pub(crate) fn add_transient_deps<
        T: Registerable,
        #[cfg(not(feature = "tokio"))] Deps: DepBuilder<T> + 'static,
        #[cfg(feature = "tokio")] Deps: DepBuilder<T> + Sync + 'static,
    >(
        &self,
    ) {
        let visitor: Visitor = |this| {
            // We already visited this type.
            if let Some(index) = this.visited.read().get(&TypeId::of::<T>()) {
                return *index;
            }

            let current = {
                let mut graph = this.graph.write();
                graph.add_node(std::any::type_name::<T>())
            };

            // We visited this type. This must be added before we visit dependencies.
            {
                let mut visited = this.visited.write();
                visited.insert(TypeId::of::<T>(), current);
            }

            let type_ids =
                Deps::as_typeids(dependency_builder::private::SealToken);

            for (type_id, type_name) in &type_ids {
                // We have been to the dependency type before, we don't need to do it again.
                if let Some(index) = this.visited.read().get(type_id) {
                    this.graph.write().add_edge(current, *index, ());
                    continue;
                }

                // Never seen the type before, visit it.
                if let Some(visitor) = this.visitor.read().get(type_id) {
                    let index = (visitor)(this);
                    this.graph.write().add_edge(current, index, ());
                    continue;
                }

                {
                    let mut missing = this.missing.write();
                    if let Some(ty) = missing.get_mut(&TypeId::of::<T>()) {
                        ty.deps.push((*type_id, type_name));
                    } else {
                        missing.insert(
                            TypeId::of::<T>(),
                            MissingDependencies {
                                ty: (
                                    TypeId::of::<T>(),
                                    std::any::type_name::<T>(),
                                ),
                                deps: vec![(*type_id, type_name)],
                            },
                        );
                    }
                }

                #[cfg(feature = "tracing")]
                tracing::warn!(
                    "couldn't add dependency of {}: {type_name}",
                    std::any::type_name::<T>()
                );
            }

            current
        };

        // TODO: Make this lock free!
        {
            let _ = self.validation_cache.write().take();
        }
        self.visitor_visited.store(false, Ordering::Release);
        self.visitor.write().insert(TypeId::of::<T>(), visitor);
    }

    /// Register a new singleton, with dependencies specified via `Deps`.
    pub(crate) fn add_singleton_deps<
        T: RegisterableSingleton,
        #[cfg(not(feature = "tokio"))] Deps: DepBuilder<T> + 'static,
        #[cfg(feature = "tokio")] Deps: DepBuilder<T> + Sync + 'static,
    >(
        &self,
    ) {
        self.add_transient_deps::<T, Deps>();
    }

    /// Walk the dependency graph and validate that all types can be constructed, all dependencies
    /// are fulfillable and there are no cycles in the graph.
    pub(crate) fn validate_all(&self) -> Result<(), ValidationError> {
        // This **must** be a separate `if`, otherwise the lock is held also in the `else`.
        if let Some(cache) = &*self.validation_cache.read() {
            eprintln!("use cache",);
            // Validation is cached.
            {
                let missing = self.missing.read();
                if missing.len() > 0 {
                    let mut vec = Vec::with_capacity(missing.len());
                    for (_, ty) in missing.iter() {
                        vec.push(ty.clone());
                    }
                    return Err(ValidationError::Missing(vec));
                }
            }

            // EARLY RETURN ABSOLUTELY REQUIRED!
            return match cache {
                Ok(_) => Ok(()),
                Err(_err) => Err(ValidationError::Cycle),
            };
        }

        // Validation is **not** cached.
        eprintln!("calculate validation",);

        if self
            .visitor_visited
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            eprintln!("VISIT!");
            // Make sure we have all types registered.
            let visitor = self.visitor.read();
            for (_type_id, cb) in visitor.iter() {
                (cb)(self);
            }
        }

        {
            let missing = self.missing.read();
            if missing.len() > 0 {
                let mut vec = Vec::with_capacity(missing.len());
                for (_, ty) in missing.iter() {
                    vec.push(ty.clone());
                }
                return Err(ValidationError::Missing(vec));
            }
        }

        let graph = self.graph.read();
        let mut space = petgraph::algo::DfsSpace::new(&*graph);
        let result = petgraph::algo::toposort(&*graph, Some(&mut space));
        let ret = match result {
            Ok(_) => Ok(()),
            Err(_) => Err(ValidationError::Cycle),
        };

        // Cache the result.
        {
            let mut cache = self.validation_cache.write();
            eprintln!("write to cache");
            *cache = Some(result);
        }

        ret
    }

    /// Validate whether the type `T` is constructible.
    pub(crate) fn validate<T>(&self) -> Result<(), ValidationError>
    where
        T: Registerable,
    {
        let _ = std::marker::PhantomData::<T>;
        self.validate_all()
    }

    /// Return a string of the dependency graph visualized using graphviz's `dot` language.
    pub(crate) fn dotgraph(&self) -> Result<String, ValidationError> {
        self.validate_all()?;

        let graph = self.graph.read();
        let dot = petgraph::dot::Dot::with_config(
            &*graph,
            &[petgraph::dot::Config::EdgeNoLabel],
        );

        Ok(format!("{dot:?}"))
    }
}