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
    /// This is a really trivial, inefficient implementation, but we assume that there
    /// are very few entries in the table.
    pub fn route(&self, hostname: &str, path: &str) -> Option<Service> {
        for (route_match, service) in self.entries.iter() {
            if route_match.matches(hostname, path) {
                return Some(service.clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use crate::route::{RouteMatchBuilder, RouteTable, Service};

    #[test]
    fn trivial_case() {
        let mut t = RouteTable::new();
        let s1 = Service::new("service1", 8080);
        let s2 = Service::new("service2", 8080);

        t.add(RouteMatchBuilder::new().hostname("example.org").build(), s1);
        t.add(
            RouteMatchBuilder::new().hostname("example.com").build(),
            s2.clone(),
        );

        let s = t.route("example.com", "/foo/bar");

        assert_eq!(Some(s2), s);
    }

    #[test]
    fn test_first_entry_wins() {
        let mut t = RouteTable::new();
        let s1 = Service::new("service1", 8080);
        let s2 = Service::new("service2", 8080);
        let s3 = Service::new("service3", 8080);

        t.add(RouteMatchBuilder::new().hostname("example.org").build(), s1);
        t.add(
            RouteMatchBuilder::new()
                .hostname("example.com")
                .path("/foo/bar")
                .build(),
            s2.clone(),
        );
        t.add(
            RouteMatchBuilder::new()
                .hostname("example.com")
                .path("/foo/bar")
                .build(),
            s3,
        );

        let s = t.route("example.com", "/foo/bar");

        assert_eq!(Some(s2), s);
    }

    #[test]
    fn test_remove_entry() {
        let mut t = RouteTable::new();
        let s1 = Service::new("service1", 8080);
        let s2 = Service::new("service2", 8080);
        let s3 = Service::new("service3", 8080);

        t.add(RouteMatchBuilder::new().hostname("example.org").build(), s1);
        t.add(
            RouteMatchBuilder::new().hostname("example2.com").build(),
            s2.clone(),
        );
        t.add(
            RouteMatchBuilder::new().hostname("example3.com").build(),
            s3,
        );
        t.add(
            RouteMatchBuilder::new().hostname("example4.com").build(),
            s2.clone(),
        );

        assert_eq!(4, t.entries.len());
        assert_eq!(Some(s2.clone()), t.route("example2.com", "/foo/bar"));
        assert_eq!(Some(s2.clone()), t.route("example4.com", "/foo/bar"));

        t.remove(s2);

        assert_eq!(2, t.entries.len());
        assert_eq!(None, t.route("example2.com", "/foo/bar"));
        assert_eq!(None, t.route("example4.com", "/foo/bar"));
    }
}
