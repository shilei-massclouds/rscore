
use core::alloc;

pub struct FILE {
}

impl FILE {
    pub fn Write(&self, s: &str) {
    }
}

pub static stdout: FILE = FILE{};
