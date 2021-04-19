use rand::random;

use crate::io::MemoryStream;
use core::fmt;

#[derive(Copy, Clone)]
pub struct AgentName(u128);

impl fmt::Display for AgentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0.to_ne_bytes()))
    }
}

impl PartialEq for AgentName {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

pub struct Agent {
    pub name: AgentName,
    pub stream: MemoryStream,
}

impl Agent {
    pub fn new() -> Self {
        Self::from_stream(MemoryStream::new())
    }

    pub fn from_stream(stream: MemoryStream) -> Agent {
        Agent {
            name: AgentName(random()),
            stream,
        }
    }
}

pub const NO_AGENT: AgentName = AgentName(u128::min_value());
