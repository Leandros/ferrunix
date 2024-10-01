use rusty_injector::Registry;

pub mod tests;

#[test]
fn runner() {
    println!("running manual tests ...");
    unsafe { Registry::reset_registry(); }
    tests::test_manual::test();

    println!("running derive tests ...");
    unsafe { Registry::reset_registry(); }
    tests::test_derive::test();
}
