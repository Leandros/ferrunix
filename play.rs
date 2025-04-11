trait IntoRes<T> {
    fn into_res(self) -> Result<T, String>;
}

impl<T> IntoRes<T> for Result<T, String> {
    fn into_res(self) -> Result<T, String> {
        self
    }
}

impl<T> IntoRes<T> for T {
    fn into_res(self) -> Result<T, String> {
        Ok(self)
    }
}

fn build<T, C, R>(ctor: C) -> Result<T, String>
where
    T: 'static,
    C: FnOnce() -> R,
    R: IntoRes<T>,
{
    let r = (ctor)();
    let x = r.into_res();
    x
}

fn main() {
    let x = build::<u8, _, _>(|| 123_u8);
    dbg!(x);
}
