#[path = "src/base/args.rs"]
mod args;

use args::Args;
use clap::CommandFactory;
use clap_complete::{generate_to, shells::Bash};
use std::env::var_os;

fn main() {
    generate_completions();
}

fn generate_completions() {
    let out_dir = var_os("OUT_DIR").unwrap();
    let mut cmd = Args::command();
    let pkg_name = env!("CARGO_PKG_NAME");

    generate_to(Bash, &mut cmd, pkg_name, &out_dir).unwrap();

    println!("cargo:rerun-if-changed=src/base/args.rs");
}
