mod matcher;
mod table;

pub use matcher::*;
pub use table::*;

/// `Service` describes an upstream application that can be routed to.
/// Right now, all we care about is the port - we assume that all applications
/// run on the same host as the agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    name: String,
    port: u16,
}
