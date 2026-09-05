mod ports;

pub use ports::{
    Clock, EmotionModel, FaceFinder, Gateway, NullFaceFinder, Queue, StubEmotionModel,
    StubGateway, StubQueue, SystemClock,
};

use serde::Serialize;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Health {
    Ok,
    Camera,
    Model,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrayStatus {
    Ativo,
    #[serde(rename = "Sem recorte")]
    SemRecorte,
    Falha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncLine {
    Pendente,
    #[allow(dead_code)]
    Ok,
    #[allow(dead_code)]
    Erro,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TraySnapshot {
    pub status: TrayStatus,
    pub sync: SyncLine,
    pub emotion_pt: Option<String>,
}

pub type LiveAgent =
    Agent<SystemClock, NullFaceFinder, StubEmotionModel, StubQueue, StubGateway>;

pub struct Agent<C, F, M, Q, G> {
    clock: C,
    face_finder: F,
    #[allow(dead_code)]
    emotion_model: M,
    #[allow(dead_code)]
    queue: Q,
    #[allow(dead_code)]
    gateway: G,
    camera_ok: bool,
    last_was_classification: bool,
}

impl LiveAgent {
    pub fn new() -> Self {
        Self::with_ports(
            SystemClock,
            NullFaceFinder,
            StubEmotionModel,
            StubQueue,
            StubGateway,
        )
    }
}

impl Default for LiveAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, F, M, Q, G> Agent<C, F, M, Q, G>
where
    C: Clock,
    F: FaceFinder,
    M: EmotionModel,
    Q: Queue,
    G: Gateway,
{
    pub fn with_ports(clock: C, face_finder: F, emotion_model: M, queue: Q, gateway: G) -> Self {
        Self {
            clock,
            face_finder,
            emotion_model,
            queue,
            gateway,
            camera_ok: true,
            last_was_classification: false,
        }
    }

    pub fn ingest_frame(&mut self, bytes: &[u8], at: SystemTime) {
        let _ocorrido_em = at;
        let _agora = self.clock.now();
        let _recorte = self.face_finder.find(bytes);
        self.last_was_classification = false;
    }

    pub fn set_camera_ok(&mut self, ok: bool) {
        self.camera_ok = ok;
    }

    #[allow(dead_code)]
    pub fn health(&self) -> Health {
        if !self.camera_ok {
            Health::Camera
        } else {
            Health::Ok
        }
    }

    #[allow(dead_code)]
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    pub fn snapshot(&self) -> TraySnapshot {
        let status = if !self.camera_ok {
            TrayStatus::Falha
        } else if self.last_was_classification {
            TrayStatus::Ativo
        } else {
            TrayStatus::SemRecorte
        };

        TraySnapshot {
            status,
            sync: SyncLine::Pendente,
            emotion_pt: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::ports::FakeClock;
    use std::time::SystemTime;

    fn agente_teste() -> Agent<FakeClock, NullFaceFinder, StubEmotionModel, StubQueue, StubGateway>
    {
        Agent::with_ports(
            FakeClock::at_unix_epoch(),
            NullFaceFinder,
            StubEmotionModel,
            StubQueue,
            StubGateway,
        )
    }

    #[test]
    fn ingest_sem_recorte_nao_persiste_frame() {
        let mut agent = agente_teste();
        let frame = [0xFFu8; 32];

        agent.ingest_frame(&frame, SystemTime::UNIX_EPOCH);

        let snap = agent.snapshot();
        assert_eq!(snap.status, TrayStatus::SemRecorte);
        assert!(snap.emotion_pt.is_none());
        assert_eq!(snap.sync, SyncLine::Pendente);
        assert_eq!(agent.pending_count(), 0);
        assert_eq!(agent.health(), Health::Ok);
    }

    #[test]
    fn camera_off_snapshot_falha() {
        let mut agent = agente_teste();
        agent.set_camera_ok(false);

        let snap = agent.snapshot();
        assert_eq!(snap.status, TrayStatus::Falha);
        assert_eq!(agent.health(), Health::Camera);
        assert!(snap.emotion_pt.is_none());
    }

    #[test]
    fn camera_ok_apos_gap_volta_sem_recorte() {
        let mut agent = agente_teste();
        agent.set_camera_ok(false);
        agent.ingest_frame(&[1, 2, 3], SystemTime::UNIX_EPOCH);
        assert_eq!(agent.snapshot().status, TrayStatus::Falha);

        agent.set_camera_ok(true);
        assert_eq!(agent.snapshot().status, TrayStatus::SemRecorte);
        assert_eq!(agent.pending_count(), 0);
    }
}
