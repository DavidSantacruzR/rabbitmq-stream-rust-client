use fake::{Fake, Faker};
use futures::StreamExt;
use rabbitmq_stream_client::types::{Message, OffsetSpecification};
use tokio::sync::mpsc::channel;

use crate::common::TestEnvironment;

#[tokio::test(flavor = "multi_thread")]
async fn producer_send_no_name_ok() {
    let env = TestEnvironment::create().await;

    let producer = env.env.producer().build(&env.stream).await.unwrap();

    let mut consumer = env
        .env
        .consumer()
        .offset(OffsetSpecification::Next)
        .build(&env.stream)
        .await
        .unwrap();

    let _ = producer
        .send_with_confirm(Message::builder().body(b"message".to_vec()).build())
        .await
        .unwrap();

    producer.close().await.unwrap();

    let delivery = consumer.next().await.unwrap().unwrap();
    assert_eq!(1, delivery.subscription_id());
    assert_eq!(Some(b"message".as_ref()), delivery.message().data());

    consumer.handle().close().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn producer_send_name_with_deduplication_ok() {
    let env = TestEnvironment::create().await;

    let mut producer = env
        .env
        .producer()
        .name("myconsumer")
        .build(&env.stream)
        .await
        .unwrap();

    let mut consumer = env
        .env
        .consumer()
        .offset(OffsetSpecification::Next)
        .build(&env.stream)
        .await
        .unwrap();

    let _ = producer
        .send_with_confirm(Message::builder().body(b"message0".to_vec()).build())
        .await
        .unwrap();

    // this is not published
    let _ = producer
        .send_with_confirm(
            Message::builder()
                .body(b"message0".to_vec())
                .publising_id(0)
                .build(),
        )
        .await
        .unwrap();

    let _ = producer
        .send_with_confirm(Message::builder().body(b"message1".to_vec()).build())
        .await
        .unwrap();

    producer.close().await.unwrap();

    let delivery = consumer.next().await.unwrap().unwrap();
    assert_eq!(1, delivery.subscription_id());
    assert_eq!(Some(b"message0".as_ref()), delivery.message().data());

    let delivery = consumer.next().await.unwrap().unwrap();
    assert_eq!(1, delivery.subscription_id());
    assert_eq!(Some(b"message1".as_ref()), delivery.message().data());

    consumer.handle().close().await.unwrap();
}
#[tokio::test(flavor = "multi_thread")]
async fn producer_send_with_callback() {
    let env = TestEnvironment::create().await;
    let reference: String = Faker.fake();

    let (tx, mut rx) = channel(1);
    let mut producer = env
        .env
        .producer()
        .name(&reference)
        .build(&env.stream)
        .await
        .unwrap();

    producer
        .send(
            Message::builder().body(b"message".to_vec()).build(),
            move |confirm_result| {
                let inner_tx = tx.clone();
                async move {
                    let _ = inner_tx.send(confirm_result).await;
                }
            },
        )
        .await
        .unwrap();

    let result = rx.recv().await.unwrap();

    assert_eq!(0, result.unwrap().publishing_id());

    producer.close().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn producer_batch_send_with_callback() {
    let env = TestEnvironment::create().await;

    let (tx, mut rx) = channel(1);
    let producer = env.env.producer().build(&env.stream).await.unwrap();

    producer
        .batch_send(
            vec![Message::builder().body(b"message".to_vec()).build()],
            move |confirm_result| {
                let inner_tx = tx.clone();
                async move {
                    let _ = inner_tx.send(confirm_result).await;
                }
            },
        )
        .await
        .unwrap();

    let result = rx.recv().await.unwrap().unwrap();

    assert_eq!(0, result.publishing_id());
    assert!(result.confirmed());
    assert_eq!(Some(b"message".as_ref()), result.message().data());
    assert!(result.message().publishing_id().is_none());

    producer.close().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn producer_batch_send() {
    let env = TestEnvironment::create().await;

    let producer = env.env.producer().build(&env.stream).await.unwrap();

    let result = producer
        .batch_send_with_confirm(vec![Message::builder().body(b"message".to_vec()).build()])
        .await
        .unwrap();

    assert_eq!(1, result.len());

    let confirmation = result.get(0).unwrap();
    assert_eq!(0, confirmation.publishing_id());
    assert!(confirmation.confirmed());
    assert_eq!(Some(b"message".as_ref()), confirmation.message().data());
    assert!(confirmation.message().publishing_id().is_none());

    producer.close().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn producer_send_with_complex_message_ok() {
    let env = TestEnvironment::create().await;

    let producer = env.env.producer().build(&env.stream).await.unwrap();

    let mut consumer = env
        .env
        .consumer()
        .offset(OffsetSpecification::Next)
        .build(&env.stream)
        .await
        .unwrap();

    let _ = producer
        .send_with_confirm(
            Message::builder()
                .body(b"message".to_vec())
                .properties()
                .message_id(32u64)
                .message_builder()
                .message_annotations()
                .insert("test", "test")
                .insert("test", true)
                .insert("test", 3u8)
                .message_builder()
                .build(),
        )
        .await
        .unwrap();

    producer.close().await.unwrap();

    let delivery = consumer.next().await.unwrap().unwrap();
    assert_eq!(1, delivery.subscription_id());

    let message = delivery.message();
    assert_eq!(Some(b"message".as_ref()), message.data());
    let properties = message.properties();

    assert_eq!(
        Some(32u64.into()),
        properties.and_then(|properties| properties.message_id.clone())
    );

    consumer.handle().close().await.unwrap();
}
