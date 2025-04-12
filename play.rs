

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

pub trait Ctor<T> {
    fn call(self) -> Result<T, String>;
}

impl<T, F, R> Ctor<T> for F
where
    F: FnOnce() -> R,
    R: IntoRes<T>,
{
    fn call(self) -> Result<T, String> {
        (self)().into_res()
    }
}

fn build<T, C>(ctor: C) -> Result<T, String>
where
    T: 'static,
    C: Ctor<T>,
{
    let r = ctor.call();
    let x = r.into_res();
    x
}

fn main() {
    let x = build::<u8, _>(|| 123_u8);
    dbg!(x);
}
