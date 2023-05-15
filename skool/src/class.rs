use std::array::TryFromSliceError;

use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use tracing::error;
use uuid::Uuid;

use crate::{error::AppError, session::Session, Result, System};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Class {
    pub school: SchoolHash,
    pub reference: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct SchoolHash(#[serde(with = "hex::serde")] [u8; 32]);

impl SchoolHash {
    pub fn new(system: System, reference: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&[system.as_u8()]);
        hasher.update(reference);
        Self(*hasher.finalize().as_bytes())
    }
}

impl TryFrom<&[u8]> for SchoolHash {
    type Error = TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

impl AsRef<[u8]> for SchoolHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub async fn from_session(session: Session) -> Result<Class> {
    match session {
        Session::Skolplattformen(session) => {
            let client = skolplattformen::Client::new(session)?;

            let timetable = crate::skolplattformen::single_timetable(&client).await?;
            let filters =
                skolplattformen::schedule::available_filters(&client, &timetable.unit_guid).await?;

            if filters.classes.len() != 1 {
                error!(classes = ?filters.classes, "expected 1 class, got {}", filters.classes.len());
                return Err(AppError::InternalError);
            }
            let class = filters.classes.into_iter().next().unwrap();

            Ok(Class {
                school: SchoolHash::new(System::Skolplattformen, timetable.unit_guid.as_bytes()),
                reference: class.group_guid,
                name: class.group_name,
            })
        }
    }
}

pub async fn add_to_class<'a>(
    class: &Class,
    uid: Uuid,
    tx: &mut Transaction<'a, Postgres>,
) -> Result<()> {
    sqlx::query!(
        r#"
          INSERT INTO classes (school, reference, name) VALUES ($1, $2, $3)
          ON CONFLICT ON CONSTRAINT classes_pkey DO UPDATE
            SET name = EXCLUDED.name
        "#,
        class.school.as_ref(),
        class.reference,
        class.name
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE credentials SET (school, class_reference) = ($1, $2) WHERE uid = $3",
        class.school.as_ref(),
        class.reference,
        uid
    )
    .execute(tx)
    .await?;

    Ok(())
}
