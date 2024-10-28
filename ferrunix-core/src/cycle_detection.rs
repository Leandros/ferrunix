//! Implementation of a cycle detection algorithm for our dependency resolution algorithm.
#![allow(clippy::missing_docs_in_private_items, missing_docs)]

use std::any::TypeId;

use crate::dependency_builder::{self, DepBuilder};
use crate::types::{HashMap, NonAsyncRwLock, Registerable, Validator};

// &mut petgraph::Graph<std::any::TypeId, (), petgraph::Directed>,
// &mut HashMap<std::any::TypeId, petgraph::graph::NodeIndex>,

pub(crate) struct DependencyValidator {
    visitor: NonAsyncRwLock<HashMap<TypeId, Validator>>,
    graph: NonAsyncRwLock<petgraph::Graph<TypeId, (), petgraph::Directed>>,
    visited: NonAsyncRwLock<HashMap<TypeId, petgraph::graph::NodeIndex>>,
    missing: NonAsyncRwLock<HashMap<TypeId, &'static str>>,
}

impl DependencyValidator {
    pub(crate) fn new() -> Self {
        Self {
            visitor: NonAsyncRwLock::new(HashMap::new()),
            graph: NonAsyncRwLock::new(petgraph::Graph::new()),
            visited: NonAsyncRwLock::new(HashMap::new()),
            missing: NonAsyncRwLock::new(HashMap::new()),
        }
    }

    pub(crate) fn add_transient_no_deps<T>(&self)
    where
        T: Registerable,
    {
        let visitor: Validator = Box::new(|this| {
            if let Some(index) = this.visited.read().get(&TypeId::of::<T>()) {
                return *index;
            }

            let index = {
                let mut graph = this.graph.write();
                graph.add_node(TypeId::of::<T>())
            };

            {
                let mut visited = this.visited.write();
                visited.insert(TypeId::of::<T>(), index);
            }

            index
        });

        self.visitor.write().insert(TypeId::of::<T>(), visitor);
    }

    pub(crate) fn add_singleton_no_deps<T>(&self)
    where
        T: Registerable,
    {
        todo!()
    }

    pub(crate) fn add_transient_deps<T, Deps>(&self)
    where
        T: Registerable,
        Deps: DepBuilder<T> + 'static,
    {
        let visitor: Validator = Box::new(|this| {
            // We already visited this type.
            if let Some(index) = this.visited.read().get(&TypeId::of::<T>()) {
                return *index;
            }

            let current = {
                let mut graph = this.graph.write();
                graph.add_node(TypeId::of::<T>())
            };

            // We visited this type. This must be added before we visit dependencies.
            {
                let mut visited = this.visited.write();
                visited.insert(TypeId::of::<T>(), current);
            }

            let type_ids =
                Deps::as_typeids(dependency_builder::private::SealToken);

            for type_id in &type_ids {
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
                    missing.insert(TypeId::of::<T>(), std::any::type_name::<T>());
                }
                eprintln!(
                    "couldn't add edge for dependency: {} {type_id:?}",
                    std::any::type_name::<T>()
                );
            }

            current
        });

        self.visitor.write().insert(TypeId::of::<T>(), visitor);
    }

    pub(crate) fn validate(&self) -> bool {
        {
            let visitor = self.visitor.read();

            // Visit all types once.
            for (_type_id, cb) in visitor.iter() {
                (cb)(self);
            }
        }

        {
            let missing = self.missing.read();
            if missing.len() > 0 {
                eprintln!("dependencies are missing for: {missing:#?}");
                return false;
            }
        }

        let graph = self.graph.read();
        dbg!(&graph);
        let mut space = petgraph::algo::DfsSpace::new(&*graph);
        let result = petgraph::algo::toposort(&*graph, Some(&mut space));
        dbg!(&result);

        result.is_ok()
    }
}
