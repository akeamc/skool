use std::env;

use skolplattformen::auth::{self, Session};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let session = auth::start_session(
        &env::var("SKOLPLATTFORMEN_USERNAME").unwrap(),
        &env::var("SKOLPLATTFORMEN_PASSWORD").unwrap(),
    )
    .await
    .unwrap();

    let key = b"bruhbruhbruhbruhbruhbruhbruhbruh";

    let ciphertext = skolplattformen::crypto::encrypt(&session, key).unwrap();

    println!("{}", ciphertext);
    println!("{} bytes", ciphertext.len());

    let plain: Session = skolplattformen::crypto::decrypt(&ciphertext, key).unwrap();
}
