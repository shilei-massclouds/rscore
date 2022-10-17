use ini::Ini;
use std::io::Write;
use std::str::FromStr;
use std::process::Command;

const KERNEL_CONFIG: &str = "kernel/config.ini";
const GENERATED_LD: &str = "kernel.generated.ld";
const GENERATED_RS: &str = "config.generated.rs";

fn prepare_params() {
    let config = Ini::load_from_file(KERNEL_CONFIG)
        .expect("Can't find {KERNEL_CONFIG}");

    let mut f_ld = std::fs::File::create(GENERATED_LD)
        .expect("Can't create {GENERATED_LD}");

    let mut f_rs = std::fs::File::create(GENERATED_RS)
        .expect("Can't create {GENERATED_RS}");

    for (_, p) in (&config).iter() {
        for (k, v) in (&p).iter() {
            writeln!(&mut f_ld, "{} = {}", k, v).unwrap();

            let ret = if v.starts_with("0x") {
                usize::from_str_radix(v.trim_start_matches("0x"), 16)
            } else {
                usize::from_str(v)
            };

            let t = match ret {
                Ok(_) => "u64",
                _ => "&str",
            };

            writeln!(&mut f_rs, "const {}: {} = {};", k, t, v).unwrap();
        }
    }
}

fn build_kernel() {
    let status = Command::new("cargo").arg("build")
        .arg("--no-default-features")
        .args(["--package", "kernel"])
        .args(["--target", "kernel/src/arch/riscv64/riscv64.json"])
		.args(["-Z", "build-std=core,alloc"])
        .args(["-Z", "build-std-features=compiler-builtins-mem"])
        .arg("--release")
        .status().unwrap();

    if !status.success() {
        panic!("Err: {}", status.code().unwrap());
    }
}

fn main() {
    /*
     * Prepare params for kernel:
     * Generate kernel.generated.ld and config.generated.rs
     * from config.ini.
     */
    prepare_params();

    /* Build kernel */
    build_kernel();
}
