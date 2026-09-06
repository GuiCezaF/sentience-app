use crate::agent::{Gateway, PortError, SyncEnvelope};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(10);

pub struct HttpGateway {
    agent: ureq::Agent,
    url: String,
}

impl HttpGateway {
    pub fn new(url: impl Into<String>) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .build()
            .new_agent();
        Self {
            agent,
            url: url.into(),
        }
    }
}

impl Gateway for HttpGateway {
    fn send(&self, envelope: &SyncEnvelope) -> Result<(), PortError> {
        self.agent
            .post(&self.url)
            .send_json(envelope)
            .map(|_| ())
            .map_err(|err| PortError::new(err.to_string()))
    }
}
