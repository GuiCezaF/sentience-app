use crate::agent::{Classification, PortError, Queue};
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SqliteQueue {
    conn: Connection,
}

impl SqliteQueue {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, PortError> {
        let conn = Connection::open(path).map_err(port_err)?;
        Self::from_connection(conn)
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self, PortError> {
        let conn = Connection::open_in_memory().map_err(port_err)?;
        Self::from_connection(conn)
    }

    fn from_connection(conn: Connection) -> Result<Self, PortError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pending_classifications (
                classification_id TEXT PRIMARY KEY,
                subject_id TEXT NOT NULL,
                occurred_at_unix INTEGER NOT NULL,
                emotion TEXT NOT NULL
            )",
            [],
        )
        .map_err(port_err)?;
        Ok(Self { conn })
    }

    #[cfg(test)]
    fn len(&self) -> Result<usize, PortError> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pending_classifications",
                [],
                |row| row.get(0),
            )
            .map_err(port_err)?;
        Ok(usize::try_from(count).unwrap_or(0))
    }
}

impl Queue for SqliteQueue {
    fn push(&mut self, classification: &Classification) -> Result<(), PortError> {
        self.conn
            .execute(
                "INSERT INTO pending_classifications
                    (classification_id, subject_id, occurred_at_unix, emotion)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    classification.classification_id.to_string(),
                    classification.subject_id,
                    occurred_at_unix(classification.occurred_at),
                    classification.emotion.wire(),
                ],
            )
            .map_err(port_err)?;
        Ok(())
    }
}

fn port_err(err: rusqlite::Error) -> PortError {
    PortError::new(err.to_string())
}

fn occurred_at_unix(at: SystemTime) -> i64 {
    at.duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Emotion;
    use std::time::SystemTime;
    use uuid::Uuid;

    fn amostra(id: Uuid) -> Classification {
        Classification {
            classification_id: id,
            subject_id: "sujeito-teste".into(),
            occurred_at: SystemTime::UNIX_EPOCH,
            emotion: Emotion::Happy,
        }
    }

    #[test]
    fn in_memory_push_incrementa_len() {
        let mut fila = SqliteQueue::in_memory().expect("fila em memória");
        assert_eq!(fila.len().expect("len"), 0);

        fila.push(&amostra(Uuid::new_v4())).expect("push");
        assert_eq!(fila.len().expect("len"), 1);

        fila.push(&amostra(Uuid::new_v4())).expect("push");
        assert_eq!(fila.len().expect("len"), 2);
    }

    #[test]
    fn in_memory_pk_duplicada_falha() {
        let mut fila = SqliteQueue::in_memory().expect("fila em memória");
        let item = amostra(Uuid::new_v4());

        fila.push(&item).expect("primeiro push");
        assert!(fila.push(&item).is_err());
        assert_eq!(fila.len().expect("len"), 1);
    }
}
