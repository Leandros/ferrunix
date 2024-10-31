//! Implementation of a cycle detection algorithm for our dependency resolution algorithm.

use std::any::TypeId;

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
    Missing,
}

impl std::fmt::Display for ValidationError {
    #[allow(clippy::use_debug)]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cycle => write!(fmt, "cycle detected!"),
            Self::Missing => write!(fmt, "dependencies missing!"),
        }
    }
}

impl std::error::Error for ValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// Detailed validation errors.
#[derive(Debug, Clone, PartialEq, Hash)]
#[non_exhaustive]
pub enum FullValidationError {
    /// A cycle between dependencies has been detected.
    Cycle(Option<String>),
    /// Dependencies are missing.
    Missing(Vec<MissingDependencies>),
}

impl std::fmt::Display for FullValidationError {
    #[allow(clippy::use_debug)]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cycle(ref node) => match node {
                Some(node) => write!(fmt, "cycle detected at {node}"),
                None => write!(fmt, "cycle detected!"),
            },
            Self::Missing(ref all_missing) => {
                writeln!(fmt, "dependencies missing:")?;

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

impl std::error::Error for FullValidationError {
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
    /// Context for visitors.
    context: NonAsyncRwLock<VisitorContext>,
}

impl DependencyValidator {
    /// Create a new dependency validator.
    pub(crate) fn new() -> Self {
        Self {
            visitor: NonAsyncRwLock::new(HashMap::new()),
            context: NonAsyncRwLock::new(VisitorContext::new()),
        }
    }

    /// Register a new transient, without any dependencies.
    pub(crate) fn add_transient_no_deps<T>(&self)
    where
        T: Registerable,
    {
        let visitor = Visitor(|_this, _visitors, context| {
            if let Some(index) = context.visited.get(&TypeId::of::<T>()) {
                return *index;
            }

            let index = context.graph.add_node(std::any::type_name::<T>());

            context.visited.insert(TypeId::of::<T>(), index);

            index
        });

        {
            let mut visitors = self.visitor.write();
            visitors.insert(TypeId::of::<T>(), visitor);
            {
                let mut context = self.context.write();
                context.reset();
            }
        }
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
        let visitor = Visitor(|this, visitors, context| {
            // We already visited this type.
            if let Some(index) = context.visited.get(&TypeId::of::<T>()) {
                return *index;
            }

            let current = context.graph.add_node(std::any::type_name::<T>());

            // We visited this type. This must be added before we visit dependencies.
            {
                context.visited.insert(TypeId::of::<T>(), current);
            }

            let type_ids =
                Deps::as_typeids(dependency_builder::private::SealToken);

            for (type_id, type_name) in &type_ids {
                // We have been to the dependency type before, we don't need to do it again.
                if let Some(index) = context.visited.get(type_id) {
                    context.graph.add_edge(current, *index, ());
                    continue;
                }

                // Never seen the type before, visit it.
                if let Some(visitor) = visitors.get(type_id) {
                    let index = (visitor.0)(this, visitors, context);
                    context.graph.add_edge(current, index, ());
                    continue;
                }

                {
                    if let Some(ty) =
                        context.missing.get_mut(&TypeId::of::<T>())
                    {
                        ty.deps.push((*type_id, type_name));
                    } else {
                        context.missing.insert(
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
        });

        {
            let mut visitors = self.visitor.write();
            visitors.insert(TypeId::of::<T>(), visitor);
            {
                let mut context = self.context.write();
                context.reset();
            }
        }
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
        let read_context = self.context.read();
        if Self::validate_context(&read_context)? {
            // Validation result is still cached.
            return Ok(());
        }

        // No validation result is cached, drop the read lock and acquire an exclusive lock to
        // update the cached validation result.
        drop(read_context);
        let visitors = self.visitor.read();
        let mut write_context = self.context.write();
        if Self::validate_context(&write_context)? {
            // Context was updated by another thread while we waited for the exclusive write lock
            // to be acquired.
            return Ok(());
        }

        // Validation did not run, we need to run it.
        self.calculate_validation(&visitors, &mut write_context);

        // Throws an error if our dependency graph is invalid.
        Self::validate_context(&write_context)?;

        Ok(())
    }

    /// Walk the dependency graph and validate that all types can be constructed, all dependencies
    /// are fulfillable and there are no cycles in the graph.
    pub(crate) fn validate_all_full(&self) -> Result<(), FullValidationError> {
        let mut context = VisitorContext::new();
        {
            let visitors = self.visitor.read();
            self.calculate_validation(&visitors, &mut context);
        }

        // Evaluate whether we want to make this available via an option? It takes ages to
        // calculate!
        // let tarjan = petgraph::algo::tarjan_scc(&context.graph);
        // dbg!(&tarjan);

        if !context.missing.is_empty() {
            let mut vec = Vec::with_capacity(context.missing.len());
            context.missing.iter().for_each(|(_, ty)| {
                vec.push(ty.clone());
            });
            return Err(FullValidationError::Missing(vec));
        }

        if let Some(cached) = &context.validation_cache {
            return match cached {
                Ok(_) => Ok(()),
                Err(err) => {
                    let index = err.node_id();
                    let node_name = context.graph.node_weight(index);
                    return Err(FullValidationError::Cycle(
                        node_name.map(|el| (*el).to_owned()),
                    ));
                }
            };
        }

        unreachable!("this is a bug")
    }

    /// Inspect `context`, and return a [`ValidationError`] if there are errors in the dependency
    /// graph.
    ///
    /// Returns `Ok(true)` if the validation result is cached.
    /// Returns `Ok(false)` if the validation result is outdated and needs to be recalculated.
    fn validate_context(
        context: &VisitorContext,
    ) -> Result<bool, ValidationError> {
        if !context.missing.is_empty() {
            return Err(ValidationError::Missing);
        }

        if let Some(cached) = &context.validation_cache {
            return match cached {
                Ok(_) => Ok(true),
                Err(_) => Err(ValidationError::Cycle),
            };
        }

        Ok(false)
    }

    /// Visit all visitors in `self.visitor`, and create the new dependency graph.
    fn calculate_validation(
        &self,
        visitors: &HashMap<TypeId, Visitor>,
        context: &mut VisitorContext,
    ) {
        {
            for (_type_id, cb) in visitors.iter() {
                // To avoid a dead lock due to other visitors needing to be called, we pass in the
                // visitors hashmap.
                (cb.0)(self, visitors, context);
            }
        }

        // We only calculate whether we have
        let mut space = petgraph::algo::DfsSpace::new(&context.graph);
        context.validation_cache =
            Some(petgraph::algo::toposort(&context.graph, Some(&mut space)));
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

        let context = self.context.read();
        let dot = petgraph::dot::Dot::with_config(
            &context.graph,
            &[petgraph::dot::Config::EdgeNoLabel],
        );

        Ok(format!("{dot:?}"))
    }
}

/// Context that's passed into every `visitor`.
pub(crate) struct VisitorContext {
    /// Dependency graph.
    graph: petgraph::Graph<&'static str, (), petgraph::Directed>,
    /// All missing dependencies.
    missing: HashMap<TypeId, MissingDependencies>,
    /// Cache of all previously visited types. To avoid infinite recursion and as an optimization.
    visited: HashMap<TypeId, petgraph::graph::NodeIndex>,
    /// Cached validation result.
    validation_cache: Option<
        Result<
            Vec<petgraph::graph::NodeIndex>,
            petgraph::algo::Cycle<petgraph::graph::NodeIndex>,
        >,
    >,
}

impl VisitorContext {
    /// Create a new default context.
    pub fn new() -> Self {
        Self {
            graph: petgraph::Graph::new(),
            missing: HashMap::new(),
            visited: HashMap::new(),
            validation_cache: None,
        }
    }

    /// Reset the context.
    pub fn reset(&mut self) {
        self.graph.clear();
        self.missing.clear();
        self.visited.clear();
        self.validation_cache = None;
    }
}
