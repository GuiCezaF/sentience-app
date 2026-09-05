mod ports;

pub use ports::{
    Clock, EmotionModel, FaceFinder, Gateway, Queue, StubGateway, SystemClock,
};

#[allow(unused_imports)]
pub use ports::PortError;

use image::{ImageFormat, RgbImage};
use serde::Serialize;
use std::cmp::Ordering;
use std::fmt;
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;

#[cfg(test)]
pub use ports::{FakeClock, FakeEmotionModel, FakeFaceFinder, FakeQueue};

pub const CONFIDENCE_FLOOR: f32 = 0.45;

pub const CROP_SIDE: u32 = 48;
pub const CROP_PIXELS: usize = (CROP_SIDE as usize) * (CROP_SIDE as usize);

pub const DEFAULT_SUBJECT_ID: &str = "local";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emotion {
    Angry,
    Happy,
    Neutral,
    Sad,
}

impl Emotion {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Angry),
            1 => Some(Self::Happy),
            2 => Some(Self::Neutral),
            3 => Some(Self::Sad),
            _ => None,
        }
    }

    pub fn wire(self) -> &'static str {
        match self {
            Self::Angry => "angry",
            Self::Happy => "happy",
            Self::Neutral => "neutral",
            Self::Sad => "sad",
        }
    }

    pub fn pt_br(self) -> &'static str {
        match self {
            Self::Angry => "Raiva",
            Self::Happy => "Feliz",
            Self::Neutral => "Neutro",
            Self::Sad => "Triste",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classification {
    pub classification_id: Uuid,
    pub subject_id: String,
    pub occurred_at: SystemTime,
    pub emotion: Emotion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    InvalidJpeg,
    #[allow(dead_code)]
    InvalidRgb,
}

#[derive(Debug, Clone)]
pub struct Frame {
    image: RgbImage,
}

impl Frame {
    pub fn decode_jpeg(bytes: &[u8]) -> Result<Self, FrameError> {
        let image = image::load_from_memory_with_format(bytes, ImageFormat::Jpeg)
            .map_err(|_| FrameError::InvalidJpeg)?
            .to_rgb8();
        Ok(Self { image })
    }

    #[allow(dead_code)]
    pub fn from_rgb(width: u32, height: u32, pixels: &[u8]) -> Result<Self, FrameError> {
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(3))
            .ok_or(FrameError::InvalidRgb)?;
        if width == 0 || height == 0 || pixels.len() != expected {
            return Err(FrameError::InvalidRgb);
        }
        let image = RgbImage::from_raw(width, height, pixels.to_vec()).ok_or(FrameError::InvalidRgb)?;
        Ok(Self { image })
    }

    pub fn size(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    pub fn as_rgb(&self) -> &RgbImage {
        &self.image
    }

    pub fn crop_face(&self, face: &FaceBox) -> FaceCrop {
        let (x, y, side) = self.square_region(face);
        let cropped = if side == 0 {
            RgbImage::from_pixel(1, 1, image::Rgb([0, 0, 0]))
        } else {
            image::imageops::crop_imm(&self.image, x, y, side, side).to_image()
        };
        let gray = image::imageops::grayscale(&cropped);
        let resized = image::imageops::resize(
            &gray,
            CROP_SIDE,
            CROP_SIDE,
            image::imageops::FilterType::Triangle,
        );
        let mut pixels = [0u8; CROP_PIXELS];
        let raw = resized.as_raw();
        let n = raw.len().min(CROP_PIXELS);
        pixels[..n].copy_from_slice(&raw[..n]);
        FaceCrop { pixels }
    }

    fn square_region(&self, face: &FaceBox) -> (u32, u32, u32) {
        let (fw, fh) = self.size();
        if fw == 0 || fh == 0 {
            return (0, 0, 0);
        }

        let x0 = face.x.min(fw.saturating_sub(1));
        let y0 = face.y.min(fh.saturating_sub(1));
        let x1 = face.x.saturating_add(face.w).min(fw).max(x0.saturating_add(1));
        let y1 = face.y.saturating_add(face.h).min(fh).max(y0.saturating_add(1));
        let bw = x1 - x0;
        let bh = y1 - y0;

        let side = bw.max(bh).max(1);
        let cx = x0 + bw / 2;
        let cy = y0 + bh / 2;
        let half = side / 2;

        let mut sx = cx.saturating_sub(half);
        let mut sy = cy.saturating_sub(half);
        if sx.saturating_add(side) > fw {
            sx = fw.saturating_sub(side);
        }
        if sy.saturating_add(side) > fh {
            sy = fh.saturating_sub(side);
        }

        (sx, sy, side.min(fw).min(fh))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaceBox {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl FaceBox {
    pub fn area(self) -> u64 {
        u64::from(self.w) * u64::from(self.h)
    }

    pub fn center_distance(self, frame_size: (u32, u32)) -> f64 {
        let face_cx = f64::from(self.x) + f64::from(self.w) / 2.0;
        let face_cy = f64::from(self.y) + f64::from(self.h) / 2.0;
        let frame_cx = f64::from(frame_size.0) / 2.0;
        let frame_cy = f64::from(frame_size.1) / 2.0;
        let dx = face_cx - frame_cx;
        let dy = face_cy - frame_cy;
        dx.hypot(dy)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaceCrop {
    pub pixels: [u8; CROP_PIXELS],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentFault {
    Model,
    Storage,
}

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

pub struct ModelPaths {
    pub ultraface: PathBuf,
    pub emotion: PathBuf,
}

#[derive(Debug)]
pub struct AgentInitError {
    message: String,
}

impl AgentInitError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for AgentInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AgentInitError {}

pub struct Agent<C, F, M, Q, G> {
    #[allow(dead_code)]
    clock: C,
    face_finder: F,
    emotion_model: M,
    queue: Q,
    #[allow(dead_code)]
    gateway: G,
    subject_id: String,
    camera_ok: bool,
    last_emotion: Option<Emotion>,
    fault: Option<AgentFault>,
}

impl<C, F, M, Q, G> Agent<C, F, M, Q, G> {
    pub fn with_ports(
        clock: C,
        face_finder: F,
        emotion_model: M,
        queue: Q,
        gateway: G,
        subject_id: impl Into<String>,
    ) -> Self {
        Self {
            clock,
            face_finder,
            emotion_model,
            queue,
            gateway,
            subject_id: subject_id.into(),
            camera_ok: true,
            last_emotion: None,
            fault: None,
        }
    }

    #[cfg(test)]
    pub fn test_queue(&self) -> &Q {
        &self.queue
    }

    #[cfg(test)]
    pub fn test_model(&self) -> &M {
        &self.emotion_model
    }

    pub fn set_camera_ok(&mut self, ok: bool) {
        self.camera_ok = ok;
    }

    pub fn snapshot(&self) -> TraySnapshot {
        let status = if !self.camera_ok || self.fault.is_some() {
            TrayStatus::Falha
        } else if self.last_emotion.is_some() {
            TrayStatus::Ativo
        } else {
            TrayStatus::SemRecorte
        };

        TraySnapshot {
            status,
            sync: SyncLine::Pendente,
            emotion_pt: self.last_emotion.map(Emotion::pt_br).map(str::to_string),
        }
    }

    #[allow(dead_code)]
    pub fn health(&self) -> Health {
        if !self.camera_ok {
            Health::Camera
        } else if self.fault.is_some() {
            Health::Model
        } else {
            Health::Ok
        }
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
    pub fn ingest_frame(&mut self, bytes: &[u8], at: SystemTime) {
        match Frame::decode_jpeg(bytes) {
            Ok(frame) => self.ingest(frame, at),
            Err(_) => self.mark_gap(),
        }
    }

    pub fn ingest(&mut self, frame: Frame, at: SystemTime) {
        let boxes = match self.face_finder.detect(&frame) {
            Ok(boxes) => boxes,
            Err(_) => {
                self.fault = Some(AgentFault::Model);
                self.last_emotion = None;
                return;
            }
        };

        let Some(face) = pick_face(&boxes, frame.size()) else {
            self.mark_gap();
            return;
        };

        let crop = frame.crop_face(&face);
        let probs = match self.emotion_model.classify(&crop) {
            Ok(probs) => probs,
            Err(_) => {
                self.fault = Some(AgentFault::Model);
                self.last_emotion = None;
                return;
            }
        };

        let (index, confidence) = argmax4(&probs);
        if confidence < CONFIDENCE_FLOOR {
            self.mark_gap();
            return;
        }

        let Some(emotion) = Emotion::from_index(index) else {
            self.mark_gap();
            return;
        };

        let classification = Classification {
            classification_id: Uuid::new_v4(),
            subject_id: self.subject_id.clone(),
            occurred_at: at,
            emotion,
        };

        match self.queue.push(&classification) {
            Ok(()) => {
                self.fault = None;
                self.last_emotion = Some(emotion);
            }
            Err(_) => {
                self.fault = Some(AgentFault::Storage);
                self.last_emotion = None;
            }
        }
    }

    #[allow(dead_code)]
    pub fn pending_count(&self) -> usize {
        self.queue.len().unwrap_or(0)
    }

    fn mark_gap(&mut self) {
        self.last_emotion = None;
    }
}

pub fn pick_face(boxes: &[FaceBox], frame_size: (u32, u32)) -> Option<FaceBox> {
    boxes.iter().copied().max_by(|a, b| match a.area().cmp(&b.area()) {
        Ordering::Equal => b
            .center_distance(frame_size)
            .partial_cmp(&a.center_distance(frame_size))
            .unwrap_or(Ordering::Equal),
        ord => ord,
    })
}

fn argmax4(probs: &[f32; 4]) -> (usize, f32) {
    let mut best_i = 0;
    let mut best = probs[0];
    for (i, &p) in probs.iter().enumerate().skip(1) {
        if p > best {
            best = p;
            best_i = i;
        }
    }
    (best_i, best)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    const SUJEITO: &str = "sujeito-teste";
    const HAPPY_ALTO: [f32; 4] = [0.05, 0.80, 0.10, 0.05];
    const ARGMAX_BAIXO: [f32; 4] = [0.31, 0.30, 0.20, 0.19];

    type AgenteFake =
        Agent<FakeClock, FakeFaceFinder, FakeEmotionModel, FakeQueue, StubGateway>;

    fn agente_com(
        finder: FakeFaceFinder,
        model: FakeEmotionModel,
        queue: FakeQueue,
    ) -> AgenteFake {
        Agent::with_ports(
            FakeClock::at_unix_epoch(),
            finder,
            model,
            queue,
            StubGateway,
            SUJEITO,
        )
    }

    fn frame_cinza(width: u32, height: u32) -> Frame {
        let pixels = vec![80u8; (width * height * 3) as usize];
        Frame::from_rgb(width, height, &pixels).expect("frame cinza")
    }

    fn frame_metades(width: u32, height: u32) -> Frame {
        let mut pixels = vec![0u8; (width * height * 3) as usize];
        for y in 0..height {
            for x in 0..width {
                if x >= width / 2 {
                    let i = ((y * width + x) * 3) as usize;
                    pixels[i] = 255;
                    pixels[i + 1] = 255;
                    pixels[i + 2] = 255;
                }
            }
        }
        Frame::from_rgb(width, height, &pixels).expect("frame metades")
    }

    fn face(x: u32, y: u32, w: u32, h: u32) -> FaceBox {
        FaceBox { x, y, w, h }
    }

    fn media_crop(crop: &FaceCrop) -> u32 {
        crop.pixels.iter().map(|&p| u32::from(p)).sum::<u32>() / CROP_PIXELS as u32
    }

    #[test]
    fn ingest_bytes_invalidos_sao_gap() {
        let mut agent = agente_com(
            FakeFaceFinder::empty(),
            FakeEmotionModel::probs(HAPPY_ALTO),
            FakeQueue::new(),
        );

        agent.ingest_frame(&[0xFF; 32], SystemTime::UNIX_EPOCH);

        let snap = agent.snapshot();
        assert_eq!(snap.status, TrayStatus::SemRecorte);
        assert!(snap.emotion_pt.is_none());
        assert_eq!(snap.sync, SyncLine::Pendente);
        assert_eq!(agent.test_queue().items.len(), 0);
        assert_eq!(agent.health(), Health::Ok);
    }

    #[test]
    fn sem_face_nao_classifica() {
        let mut agent = agente_com(
            FakeFaceFinder::empty(),
            FakeEmotionModel::probs(HAPPY_ALTO),
            FakeQueue::new(),
        );

        agent.ingest(frame_cinza(32, 32), SystemTime::UNIX_EPOCH);

        let snap = agent.snapshot();
        assert_eq!(snap.status, TrayStatus::SemRecorte);
        assert!(snap.emotion_pt.is_none());
        assert_eq!(agent.test_queue().items.len(), 0);
        assert!(agent.test_model().received_crops().is_empty());
        assert_eq!(agent.health(), Health::Ok);
    }

    #[test]
    fn argmax_abaixo_do_piso_e_gap() {
        let mut agent = agente_com(
            FakeFaceFinder::boxes(vec![face(0, 0, 16, 16)]),
            FakeEmotionModel::probs(ARGMAX_BAIXO),
            FakeQueue::new(),
        );

        agent.ingest(frame_cinza(32, 32), SystemTime::UNIX_EPOCH);

        let snap = agent.snapshot();
        assert_eq!(snap.status, TrayStatus::SemRecorte);
        assert!(snap.emotion_pt.is_none());
        assert_eq!(agent.test_queue().items.len(), 0);
        assert_eq!(agent.health(), Health::Ok);
    }

    #[test]
    fn happy_acima_do_piso_persiste_uma_classificacao() {
        let mut agent = agente_com(
            FakeFaceFinder::boxes(vec![face(0, 0, 16, 16)]),
            FakeEmotionModel::probs(HAPPY_ALTO),
            FakeQueue::new(),
        );
        let at = SystemTime::UNIX_EPOCH;

        agent.ingest(frame_cinza(32, 32), at);

        let snap = agent.snapshot();
        assert_eq!(snap.status, TrayStatus::Ativo);
        assert_eq!(snap.emotion_pt.as_deref(), Some("Feliz"));
        assert_eq!(agent.test_queue().items.len(), 1);

        let item = &agent.test_queue().items[0];
        assert_eq!(item.subject_id, SUJEITO);
        assert_eq!(item.occurred_at, at);
        assert_eq!(item.emotion, Emotion::Happy);
        assert_eq!(agent.health(), Health::Ok);
    }

    #[test]
    fn duas_faces_usa_a_maior_e_uma_classificacao() {
        let mut agent = agente_com(
            FakeFaceFinder::boxes(vec![face(0, 0, 8, 8), face(32, 0, 32, 32)]),
            FakeEmotionModel::probs(HAPPY_ALTO),
            FakeQueue::new(),
        );

        agent.ingest(frame_metades(64, 32), SystemTime::UNIX_EPOCH);

        assert_eq!(agent.test_queue().items.len(), 1);
        assert_eq!(agent.test_model().received_crops().len(), 1);
        let mean = media_crop(&agent.test_model().last_crop().expect("crop"));
        assert!(mean > 200, "recorte deveria vir da metade clara, mean={mean}");
        assert_eq!(agent.snapshot().status, TrayStatus::Ativo);
    }

    #[test]
    fn empate_de_area_escolhe_a_mais_central() {
        let frame = (100, 100);
        let esquerda = face(0, 40, 20, 20);
        let centro = face(40, 40, 20, 20);
        assert_eq!(esquerda.area(), centro.area());
        assert_eq!(pick_face(&[esquerda, centro], frame), Some(centro));
        assert_eq!(pick_face(&[centro, esquerda], frame), Some(centro));
    }

    #[test]
    fn pick_face_vazio_e_maior_area() {
        assert!(pick_face(&[], (32, 32)).is_none());
        let pequena = face(0, 0, 4, 4);
        let grande = face(10, 10, 20, 20);
        assert_eq!(pick_face(&[pequena, grande], (40, 40)), Some(grande));
    }

    #[test]
    fn crop_face_produz_48x48() {
        let frame = frame_cinza(20, 20);
        let crop = frame.crop_face(&face(2, 2, 8, 8));
        assert_eq!(crop.pixels.len(), CROP_PIXELS);
    }

    #[test]
    fn falha_do_modelo_e_proximo_tick_ok_limpa() {
        let mut agent = agente_com(
            FakeFaceFinder::boxes(vec![face(0, 0, 16, 16)]),
            FakeEmotionModel::fail(),
            FakeQueue::new(),
        );
        let frame = frame_cinza(32, 32);

        agent.ingest(frame.clone(), SystemTime::UNIX_EPOCH);
        assert_eq!(agent.snapshot().status, TrayStatus::Falha);
        assert_eq!(agent.health(), Health::Model);
        assert_eq!(agent.test_queue().items.len(), 0);
        assert!(agent.snapshot().emotion_pt.is_none());

        agent.test_model().set_probs(HAPPY_ALTO);
        agent.ingest(frame, SystemTime::UNIX_EPOCH);
        assert_eq!(agent.snapshot().status, TrayStatus::Ativo);
        assert_eq!(agent.health(), Health::Ok);
        assert_eq!(agent.snapshot().emotion_pt.as_deref(), Some("Feliz"));
        assert_eq!(agent.test_queue().items.len(), 1);
    }

    #[test]
    fn falha_da_fila_nao_anuncia_ativo() {
        let mut agent = agente_com(
            FakeFaceFinder::boxes(vec![face(0, 0, 16, 16)]),
            FakeEmotionModel::probs(HAPPY_ALTO),
            FakeQueue::fail_on_push(),
        );

        agent.ingest(frame_cinza(32, 32), SystemTime::UNIX_EPOCH);

        let snap = agent.snapshot();
        assert_eq!(snap.status, TrayStatus::Falha);
        assert!(snap.emotion_pt.is_none());
        assert_eq!(agent.test_queue().items.len(), 0);
        assert_eq!(agent.health(), Health::Model);
    }

    #[test]
    fn classification_nao_carrega_frame() {
        let item = Classification {
            classification_id: Uuid::new_v4(),
            subject_id: SUJEITO.into(),
            occurred_at: SystemTime::UNIX_EPOCH,
            emotion: Emotion::Happy,
        };
        let dbg = format!("{item:?}");
        assert!(dbg.contains("classification_id"));
        assert!(dbg.contains("subject_id"));
        assert!(dbg.contains("occurred_at"));
        assert!(dbg.contains("emotion"));
        assert!(!dbg.contains("Frame"));
        assert!(!dbg.contains("pixels"));
        assert!(std::mem::size_of::<Classification>() < 128);
    }

    #[test]
    fn camera_off_snapshot_falha() {
        let mut agent = agente_com(
            FakeFaceFinder::empty(),
            FakeEmotionModel::probs(HAPPY_ALTO),
            FakeQueue::new(),
        );
        agent.set_camera_ok(false);

        let snap = agent.snapshot();
        assert_eq!(snap.status, TrayStatus::Falha);
        assert_eq!(agent.health(), Health::Camera);
        assert!(snap.emotion_pt.is_none());
    }

    #[test]
    fn camera_ok_apos_gap_volta_sem_recorte() {
        let mut agent = agente_com(
            FakeFaceFinder::empty(),
            FakeEmotionModel::probs(HAPPY_ALTO),
            FakeQueue::new(),
        );
        agent.set_camera_ok(false);
        agent.ingest_frame(&[1, 2, 3], SystemTime::UNIX_EPOCH);
        assert_eq!(agent.snapshot().status, TrayStatus::Falha);

        agent.set_camera_ok(true);
        assert_eq!(agent.snapshot().status, TrayStatus::SemRecorte);
        assert_eq!(agent.pending_count(), 0);
    }
}
