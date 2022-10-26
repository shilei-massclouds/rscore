/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::fmt::{Result, Write};

pub struct FILE {
}

impl Write for FILE {
    fn write_str(&mut self, s: &str) -> Result {
        for ch in s.chars() {
            crate::console_putchar(ch);
        }
        Ok(())
    }
}

pub static mut STDOUT: FILE = FILE{};
