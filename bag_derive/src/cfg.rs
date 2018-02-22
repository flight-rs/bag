use std::sync::Mutex;
use bagger::Bagger;

lazy_static! {
    pub static BAGGER: Mutex<Bagger> = Mutex::new(Bagger::new());
}
