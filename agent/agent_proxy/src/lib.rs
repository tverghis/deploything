use std::collections::HashMap;

use tracing::warn;

#[derive(Debug)]
struct Service {
    name: String,
    port: u16,
}

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

struct RouteTable {
    map: HashMap<RouteMatch, Service>,
}
