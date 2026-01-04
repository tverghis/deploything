use std::collections::HashMap;

use tracing::warn;

/// `Service` describes an upstream application that can be routed to.
/// Right now, all we care about is the port - we assume that all applications
/// run on the same host as the agent.
#[derive(Debug)]
struct Service {
    name: String,
    port: u16,
}

/// A `RouteMatch` determines the criteria by which an incoming request should be routed.
/// A request must match _both_ criteria specified in order the `RouteMatch` to be considered matched.
/// It is technically not an error if a `RouteMatch` has both `hostname` and `path` set to `None`, but
/// such an instance is functionally useless.
#[derive(Debug, Hash, PartialEq, Eq)]
struct RouteMatch {
    hostname: Option<String>,
    path: Option<String>,
}

#[derive(Debug, Default)]
struct RouteMatchBuilder {
    hostname: Option<String>,
    path: Option<String>,
}

impl RouteMatchBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(mut self) -> RouteMatch {
        let mut matcher = RouteMatch {
            hostname: None,
            path: None,
        };

        if matches!((&self.hostname, &self.path), (None, None)) {
            warn!("Building a RouteMatch with None hostname and path!");
        }

        if let Some(h) = self.hostname.take() {
            matcher.hostname = Some(h);
        }

        if let Some(p) = self.path.take() {
            matcher.path = Some(p);
        }

        matcher
    }
}

/// A `RouteTable` determines which `Service` should received the proxied request
/// based on specified `RouteMatch`es.
struct RouteTable {
    map: HashMap<RouteMatch, Service>,
}
