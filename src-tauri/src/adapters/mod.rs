mod http_gateway;
mod sqlite_queue;
mod tract_emotion;
mod ultraface;

pub use http_gateway::HttpGateway;
pub use sqlite_queue::SqliteQueue;
pub use tract_emotion::TractEmotionModel;
pub use ultraface::UltraFaceFinder;

use crate::agent::{Agent, AgentInitError, Gateway, ModelPaths, PortError, StubGateway, SyncEnvelope};
use std::path::Path;

pub enum LiveGateway {
    Http(HttpGateway),
    Stub(StubGateway),
}

impl Gateway for LiveGateway {
    fn send(&self, envelope: &SyncEnvelope) -> Result<(), PortError> {
        match self {
            Self::Http(gateway) => gateway.send(envelope),
            Self::Stub(gateway) => gateway.send(envelope),
        }
    }
}

pub type LiveAgent = Agent<UltraFaceFinder, TractEmotionModel, SqliteQueue, LiveGateway>;

impl LiveAgent {
    pub fn make(
        paths: ModelPaths,
        queue_path: impl AsRef<Path>,
        subject_id: impl Into<String>,
        gateway: LiveGateway,
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
            gateway,
            subject_id,
        ))
    }
}
