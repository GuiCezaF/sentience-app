use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaceCrop;

pub trait Clock {
    fn now(&self) -> SystemTime;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub struct FakeClock {
    now: SystemTime,
}

#[cfg(test)]
impl FakeClock {
    pub fn new(now: SystemTime) -> Self {
        Self { now }
    }

    pub fn at_unix_epoch() -> Self {
        Self::new(SystemTime::UNIX_EPOCH)
    }
}

#[cfg(test)]
impl Clock for FakeClock {
    fn now(&self) -> SystemTime {
        self.now
    }
}

pub trait FaceFinder {
    fn find(&self, frame: &[u8]) -> Option<FaceCrop>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NullFaceFinder;

impl FaceFinder for NullFaceFinder {
    fn find(&self, _frame: &[u8]) -> Option<FaceCrop> {
        None
    }
}
pub trait EmotionModel {}

#[derive(Debug, Default, Clone, Copy)]
pub struct StubEmotionModel;

impl EmotionModel for StubEmotionModel {}

pub trait Queue {
    #[allow(dead_code)]
    fn len(&self) -> usize;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StubQueue;

impl Queue for StubQueue {
    fn len(&self) -> usize {
        0
    }
}

pub trait Gateway {}

#[derive(Debug, Default, Clone, Copy)]
pub struct StubGateway;

impl Gateway for StubGateway {}
