/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

#![allow(non_camel_case_types)]

use crate::paddr_t;

pub struct vm_page_t {
    /* read-only after being set up */
    paddr_priv: paddr_t,  /* use paddr() accessor */
}
