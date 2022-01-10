use std::env;

use chrono::{Datelike, Utc};
use skolplattformen::{
    auth::{self, Session},
    schedule::{get_scope, lessons_by_week, list_timetables, ScheduleCredentials},
};

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

    let _plain: Session = skolplattformen::crypto::decrypt(&ciphertext, key).unwrap();

    let client = session.into_client();

    let scope = get_scope(&client).await.unwrap();

    let creds = ScheduleCredentials { scope };

    let timetables = list_timetables(&client, &creds).await.unwrap();

    let timetable = timetables.into_iter().next().unwrap();

    dbg!(&timetable.school_id);

    let lessons = lessons_by_week(&client, &creds, &timetable, Utc::now().iso_week())
        .await
        .unwrap();

    dbg!(lessons);
}
