pub mod deploything {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/deploything.v1.rs"));
    }
}
