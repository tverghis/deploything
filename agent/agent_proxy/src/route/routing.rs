use tracing::warn;

use crate::route::Service;

/// A `RouteMatch` determines the criteria by which an incoming request should be routed.
/// A request must match _both_ criteria specified in order the `RouteMatch` to be considered matched.
/// It is technically not an error if a `RouteMatch` has both `hostname` and `path` set to `None`, but
/// such an instance is functionally useless.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct RouteMatch {
    hostname: Option<String>,
    path: Option<String>,
}

impl RouteMatch {
    pub fn matches(&self, hostname: &str, path: &str) -> bool {
        match (&self.hostname, &self.path) {
            (None, None) => false,
            (Some(h), None) => h == hostname,
            (None, Some(p)) => p == path,
            (Some(h), Some(p)) => p == path && h == hostname,
        }
    }
}

#[derive(Debug, Default)]
pub struct RouteMatchBuilder {
    hostname: Option<String>,
    path: Option<String>,
}

impl RouteMatchBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hostname(mut self, hostname: &str) -> Self {
        self.hostname = Some(hostname.to_string());
        self
    }

    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
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
#[derive(Debug, Default)]
pub struct RouteTable {
    entries: Vec<(RouteMatch, Service)>,
}

impl RouteTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, route_match: RouteMatch, service: Service) {
        self.entries.push((route_match, service));
    }

    pub fn remove(&mut self, service: Service) {
        self.entries.retain(|(_, s)| *s != service);
    }

    /// `route` finds the first entry that matches the given hostname and path.
    /// Oldest matching entry wins.
    pub fn route(&self, hostname: &str, path: &str) -> Option<Service> {
        for (route_match, service) in self.entries.iter() {
            if route_match.matches(hostname, path) {
                return Some(service.clone());
            }
        }

        None
    }
}
