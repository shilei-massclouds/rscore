/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

pub enum Error {
    /* Indicates an operation was successful. */
    OK,

    /* The operation failed because the current state of the object
     * does not allow it, or a precondition of the operation
     * is not satisfied. */
    BadState,
}
