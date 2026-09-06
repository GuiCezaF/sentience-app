use super::{Classification, FaceBox, FaceCrop, Frame};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortError(pub String);

impl PortError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for PortError {}

pub trait FaceFinder {
    fn detect(&self, frame: &Frame) -> Result<Vec<FaceBox>, PortError>;
}

pub trait EmotionModel {
    fn classify(&self, crop: &FaceCrop) -> Result<[f32; 4], PortError>;
}

pub trait Queue {
    fn push(&mut self, classification: &Classification) -> Result<(), PortError>;
}

pub trait Gateway {}

#[derive(Debug, Default, Clone, Copy)]
pub struct StubGateway;

impl Gateway for StubGateway {}

#[cfg(test)]
#[derive(Debug, Clone)]
pub struct FakeFaceFinder {
    result: Result<Vec<FaceBox>, PortError>,
}

#[cfg(test)]
impl FakeFaceFinder {
    pub fn boxes(boxes: Vec<FaceBox>) -> Self {
        Self {
            result: Ok(boxes),
        }
    }

    pub fn empty() -> Self {
        Self::boxes(Vec::new())
    }
}

#[cfg(test)]
impl FaceFinder for FakeFaceFinder {
    fn detect(&self, _frame: &Frame) -> Result<Vec<FaceBox>, PortError> {
        self.result.clone()
    }
}

#[cfg(test)]
#[derive(Debug)]
pub struct FakeEmotionModel {
    result: std::cell::RefCell<Result<[f32; 4], PortError>>,
    received: std::cell::RefCell<Vec<FaceCrop>>,
}

#[cfg(test)]
impl FakeEmotionModel {
    pub fn probs(probs: [f32; 4]) -> Self {
        Self {
            result: std::cell::RefCell::new(Ok(probs)),
            received: std::cell::RefCell::new(Vec::new()),
        }
    }

    pub fn fail() -> Self {
        Self {
            result: std::cell::RefCell::new(Err(PortError::new("classify"))),
            received: std::cell::RefCell::new(Vec::new()),
        }
    }

    pub fn set_probs(&self, probs: [f32; 4]) {
        *self.result.borrow_mut() = Ok(probs);
    }

    pub fn last_crop(&self) -> Option<FaceCrop> {
        self.received.borrow().last().copied()
    }

    pub fn received_crops(&self) -> Vec<FaceCrop> {
        self.received.borrow().clone()
    }
}

#[cfg(test)]
impl EmotionModel for FakeEmotionModel {
    fn classify(&self, crop: &FaceCrop) -> Result<[f32; 4], PortError> {
        self.received.borrow_mut().push(*crop);
        self.result.borrow().clone()
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct FakeQueue {
    pub items: Vec<Classification>,
    push_error: Option<PortError>,
}

#[cfg(test)]
impl FakeQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fail_on_push() -> Self {
        Self {
            items: Vec::new(),
            push_error: Some(PortError::new("push")),
        }
    }
}

#[cfg(test)]
impl Queue for FakeQueue {
    fn push(&mut self, classification: &Classification) -> Result<(), PortError> {
        if let Some(error) = &self.push_error {
            return Err(error.clone());
        }
        self.items.push(classification.clone());
        Ok(())
    }
}
