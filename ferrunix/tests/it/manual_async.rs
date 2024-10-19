use ferrunix::{Registry, Transient};

#[tokio::test]
async fn test_simple() {
    let registry = Registry::empty();
    registry
        .transient_async(|| Box::pin(async move { 1_u32 }))
        .await;
    registry
        .with_deps::<_, (Transient<u32>,)>()
        .transient_async(|(x,)| {
            Box::pin(async move {
                let x = x.get();
                u64::from(x) + 2
            })
        })
        .await;

    let val = registry.get_transient_async::<u32>().await.unwrap();
    assert_eq!(val, 1);
}
