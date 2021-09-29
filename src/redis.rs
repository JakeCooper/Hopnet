use redis::AsyncCommands;

mod client;

pub fn redis() {
    let client = client::redis().await;

    let mut conn = client.get_async_connection().await.unwrap();

    let _: () = conn.set("rashiq", "testing").await.unwrap();

    let val: Option<String> = conn.get("key").await.unwrap();

    match val {
        Some(s) => println!("Value is {}", s),
        _ => (),
    };
}