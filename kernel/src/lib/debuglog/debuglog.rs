/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

/* debug print levels */
pub const ALWAYS    : u32 = 0;
pub const CRITICAL  : u32 = 0;
pub const WARN      : u32 = 1;
pub const INFO      : u32 = 2;
pub const SPEW      : u32 = 3;

pub const DEBUG_PRINT_LEVEL: u32 = SPEW;

#[macro_export]
macro_rules! dprint {
    ($level: expr, $($arg:tt)*) => {
        if $level <= crate::DEBUG_PRINT_LEVEL {
            crate::lib::libc::print::
                vprint(core::format_args!($($arg)*));
        }
    }
}

/*
 * Called first thing in init,
 * so very early printfs can go to serial console.
 */
pub fn dlog_init_early() {
}
