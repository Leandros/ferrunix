pub mod tests;

#[test]
fn runner() {
    println!("running manual tests ...");
    tests::test_manual::test();

    println!("running derivice tests ...");
    tests::test_derive::test();
}
