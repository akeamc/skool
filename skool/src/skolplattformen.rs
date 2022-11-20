use skolplattformen::schedule::list_timetables;
use tracing::error;

use crate::{error::AppError, Result};

pub async fn single_timetable(
    client: &skolplattformen::Client,
) -> Result<skolplattformen::schedule::Timetable> {
    let mut timetables = list_timetables(client).await?.into_iter();
    let timetable = timetables.next().ok_or_else(|| {
        error!("got 0 timetables");
        AppError::InternalError
    })?;
    if timetables.next().is_some() {
        error!("got more than one timetable");
        return Err(AppError::InternalError);
    }
    Ok(timetable)
}
