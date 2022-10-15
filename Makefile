# Use of this source code is governed by a MIT-style license
# that can be found in the LICENSE file or
# at https://opensource.org/licenses/MIT

all:
	cargo build --package rscore --no-default-features \
		--target src/kernel/arch/riscv64/riscv64.json \
		-Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem \
		--release
	riscv64-linux-gnu-objcopy -O binary \
		-R .note -R .note.gnu.build-id -R .comment -S \
		target/riscv64/release/rscore ./target/riscv64/release/rscore.bin
