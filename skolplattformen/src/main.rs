use std::env;

use skolplattformen::auth;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let session = auth::start_session(
        &env::var("SKOLPLATTFORMEN_USERNAME").unwrap(),
        &env::var("SKOLPLATTFORMEN_PASSWORD").unwrap(),
    )
    .await
    .unwrap();

    let json = serde_json::to_string(&session).unwrap();

    println!("{}", json);
}
