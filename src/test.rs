type StdAny = dyn std::any::Any + Send + Sync;

pub trait DepBuilder<R> {
    fn build(self, ctor: fn(Self) -> R) -> R;
}

impl<R: Send + Sync + 'static> DepBuilder<R> for () {
    fn build(self, ctor: fn(Self) -> R) -> R {
        ctor(self)
    }
}

impl<R: Send + Sync + 'static, T> DepBuilder<R> for (T,) {
    fn build(self, ctor: fn(Self) -> R) -> R {
        ctor(self)
    }
}

impl<R: Send + Sync + 'static, T1, T2> DepBuilder<R> for (T1, T2) {
    fn build(self, ctor: fn(Self) -> R) -> R {
        ctor(self)
    }
}

impl<R: Send + Sync + 'static, T1, T2, T3> DepBuilder<R> for (T1, T2, T3) {
    fn build(self, ctor: fn(Self) -> R) -> R {
        ctor(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {

    }
}
