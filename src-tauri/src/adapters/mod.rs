mod sqlite_queue;
mod tract_emotion;
mod ultraface;

pub use sqlite_queue::SqliteQueue;
pub use tract_emotion::TractEmotionModel;
pub use ultraface::UltraFaceFinder;

use crate::agent::{Agent, AgentInitError, ModelPaths, StubGateway};
use std::path::Path;

pub type LiveAgent = Agent<UltraFaceFinder, TractEmotionModel, SqliteQueue, StubGateway>;

impl LiveAgent {
    pub fn make(
        paths: ModelPaths,
        queue_path: impl AsRef<Path>,
        subject_id: impl Into<String>,
    ) -> Result<Self, AgentInitError> {
        let face_finder = UltraFaceFinder::load(&paths.ultraface).map_err(|e| {
            AgentInitError::new(format!(
                "Falha ao carregar UltraFace ({}): {e}",
                paths.ultraface.display()
            ))
        })?;
        let emotion_model = TractEmotionModel::load(&paths.emotion).map_err(|e| {
            AgentInitError::new(format!(
                "Falha ao carregar DS-CNN ({}): {e}",
                paths.emotion.display()
            ))
        })?;
        let queue = SqliteQueue::open(queue_path.as_ref()).map_err(|e| {
            AgentInitError::new(format!(
                "Falha ao abrir a fila SQLite ({}): {e}",
                queue_path.as_ref().display()
            ))
        })?;
        Ok(Self::with_ports(
            face_finder,
            emotion_model,
            queue,
            StubGateway,
            subject_id,
        ))
    }
}
