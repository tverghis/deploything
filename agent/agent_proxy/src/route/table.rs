use crate::route::{RouteMatch, Service};

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

    /// Finds the first (i.e., oldest) entry that matches the given hostname and path.
    pub fn route(&self, hostname: &str, path: &str) -> Option<Service> {
        for (route_match, service) in self.entries.iter() {
            if route_match.matches(hostname, path) {
                return Some(service.clone());
            }
        }

        None
    }
}
