/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::config_generated::*;

pub const PAGE_SIZE: usize = 1 << _CONFIG_PAGE_SIZE_SHIFT;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;
