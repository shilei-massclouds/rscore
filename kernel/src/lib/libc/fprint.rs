/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::fmt;
use core::fmt::Write;
use super::stdio::FILE;

pub fn vfprint(out: &mut FILE, args: fmt::Arguments) {
    out.write_fmt(args).unwrap();
}
