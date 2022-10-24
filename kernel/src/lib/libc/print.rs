
use core::fmt;
use super::fprint;
use super::stdio::stdout;

pub fn vprint(args: fmt::Arguments) {
    fprint::vfprint(&stdout, args);
}
