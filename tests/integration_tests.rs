#![cfg_attr(all(feature = "benchmarks", test), feature(test))]
#[cfg(all(feature = "benchmarks", test))]
extern crate test;

mod common;

#[tokio::test]
async fn test_deposit_withdrawal() {
    let file_name = "deposit_withdrawal".to_string();
    common::test_csv_data(file_name.clone()).await;
    common::test_service_from_csv_data(file_name).await;
}

#[tokio::test]
async fn test_dispute() {
    let file_name = "dispute".to_string();
    common::test_csv_data(file_name.clone()).await;
    common::test_service_from_csv_data(file_name).await;
}

#[tokio::test]
async fn test_dispute_resolve() {
    let file_name = "dispute_resolve".to_string();
    common::test_csv_data(file_name.clone()).await;
    common::test_service_from_csv_data(file_name).await;
}

#[tokio::test]
async fn test_dispute_chargeback() {
    let file_name = "dispute_chargeback".to_string();
    common::test_csv_data(file_name.clone()).await;
    common::test_service_from_csv_data(file_name).await;
}

#[cfg(feature = "benchmarks")]
#[bench]
fn bench_chungus(b: &mut ::test::Bencher) {
    let rt = rocket::tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    b.iter(|| rt.block_on(async { common::test_service_from_csv_data_concurently("chungus".to_string()).await }));
}
