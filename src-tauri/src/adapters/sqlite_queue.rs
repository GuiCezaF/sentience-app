use crate::agent::{Classification, Emotion, PortError, Queue};
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

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

    fn pending(&self) -> Result<Vec<Classification>, PortError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT classification_id, subject_id, occurred_at_unix, emotion
                 FROM pending_classifications
                 ORDER BY occurred_at_unix, classification_id",
            )
            .map_err(port_err)?;
        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let subject_id: String = row.get(1)?;
                let unix: i64 = row.get(2)?;
                let emotion: String = row.get(3)?;
                Ok((id, subject_id, unix, emotion))
            })
            .map_err(port_err)?;

        let mut items = Vec::new();
        for row in rows {
            let (id, subject_id, unix, emotion) = row.map_err(port_err)?;
            let classification_id = Uuid::parse_str(&id).map_err(|err| PortError::new(err.to_string()))?;
            let emotion = Emotion::from_wire(&emotion)
                .ok_or_else(|| PortError::new(format!("emoção inválida: {emotion}")))?;
            items.push(Classification {
                classification_id,
                subject_id,
                occurred_at: unix_to_system(unix),
                emotion,
            });
        }
        Ok(items)
    }

    fn remove(&mut self, ids: &[Uuid]) -> Result<(), PortError> {
        let tx = self.conn.transaction().map_err(port_err)?;
        for id in ids {
            tx.execute(
                "DELETE FROM pending_classifications WHERE classification_id = ?1",
                params![id.to_string()],
            )
            .map_err(port_err)?;
        }
        tx.commit().map_err(port_err)?;
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

fn unix_to_system(unix: i64) -> SystemTime {
    match u64::try_from(unix) {
        Ok(secs) => UNIX_EPOCH + Duration::from_secs(secs),
        Err(_) => UNIX_EPOCH,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Emotion;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use uuid::Uuid;

    fn amostra(id: Uuid) -> Classification {
        Classification {
            classification_id: id,
            subject_id: "sujeito-teste".into(),
            occurred_at: SystemTime::UNIX_EPOCH,
            emotion: Emotion::Happy,
        }
    }

    fn amostra_em(id: Uuid, at: SystemTime, emotion: Emotion) -> Classification {
        Classification {
            classification_id: id,
            subject_id: "sujeito-teste".into(),
            occurred_at: at,
            emotion,
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

    #[test]
    fn pending_devolve_em_ordem() {
        let mut fila = SqliteQueue::in_memory().expect("fila em memória");
        let t0 = UNIX_EPOCH + Duration::from_secs(10);
        let t1 = UNIX_EPOCH + Duration::from_secs(20);
        let id_a = Uuid::from_u128(2);
        let id_b = Uuid::from_u128(1);
        let id_c = Uuid::from_u128(3);

        fila.push(&amostra_em(id_a, t1, Emotion::Sad)).expect("push a");
        fila.push(&amostra_em(id_c, t0, Emotion::Happy)).expect("push c");
        fila.push(&amostra_em(id_b, t0, Emotion::Angry)).expect("push b");

        let pending = fila.pending().expect("pending");
        let ids: Vec<Uuid> = pending.iter().map(|item| item.classification_id).collect();
        assert_eq!(ids, vec![id_b, id_c, id_a]);
        assert_eq!(pending[0].emotion, Emotion::Angry);
        assert_eq!(pending[1].emotion, Emotion::Happy);
        assert_eq!(pending[2].emotion, Emotion::Sad);
    }

    #[test]
    fn remove_apaga_so_os_ids_passados() {
        let mut fila = SqliteQueue::in_memory().expect("fila em memória");
        let a = amostra(Uuid::from_u128(1));
        let b = amostra(Uuid::from_u128(2));
        let c = amostra(Uuid::from_u128(3));
        fila.push(&a).expect("push a");
        fila.push(&b).expect("push b");
        fila.push(&c).expect("push c");

        fila.remove(&[a.classification_id, c.classification_id])
            .expect("remove");
        let pending = fila.pending().expect("pending");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].classification_id, b.classification_id);
    }
}
