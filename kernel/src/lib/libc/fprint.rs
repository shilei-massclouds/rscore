
use core::fmt;
use super::stdio::FILE;

pub fn vfprint(out: &FILE, args: fmt::Arguments) {
    if let Some(s) = args.as_str() {
        out.Write(s);
    } else {
        out.Write("NO implementation!");
    }
}
