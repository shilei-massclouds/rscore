/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::fmt;
use super::fprint;
use super::stdio::STDOUT;

pub fn vprint(args: fmt::Arguments) {
    unsafe {
        fprint::vfprint(&mut STDOUT, args);
    }
}
