use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("lock couldn't be acquired")]
    LockAcquire,
    #[error("couldn't resolve dependencies")]
    DependenciesMissing,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("already a default registry set for this process")]
    DefaultAlreadySet,
}
