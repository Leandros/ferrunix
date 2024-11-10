#![allow(dead_code)]
use ferrunix::Inject;

#[derive(Inject)]
#[provides(singleton)]
pub struct ServerConfig {}

#[derive(Inject)]
#[provides(singleton)]
pub struct ServerConfig1 {
    #[inject(default)]
    pub hostname: String,
}
