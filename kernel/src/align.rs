/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

#[macro_export]
macro_rules! ROUNDUP {
    ($a: expr, $b: expr) => {((($a) + (($b)-1)) & !(($b)-1))}
}

#[macro_export]
macro_rules! ALIGN {
    ($a: expr, $b: expr) => {ROUNDUP!($a, $b)}
}
