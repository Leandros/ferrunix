use rusty_injector_macros::Inject;

#[derive(Debug, Inject)]
pub struct Test1 {
    #[inject]
    pub bar: i32,
}
