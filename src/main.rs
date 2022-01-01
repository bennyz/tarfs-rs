use std::path::Path;

use clap::Parser;
use env_logger::Env;
use fuser::MountOption;
use tarfs_rs::TarFs;

#[derive(Parser, Clone)]
#[clap(version = "0.0.1")]
struct Args {
    /// Tar file to mount
    file: String,

    /// Mount point
    mount: String,
}
fn main() {
    let args = Args::parse();
    if !Path::exists(Path::new(&args.file)) {
        panic!("tar file {} does not exist!", args.file);
    }

    if !Path::exists(Path::new(&args.mount)) {
        panic!("mount point {} does not exist!", args.mount);
    }

    let env = Env::default().filter_or("RUST_LOG", "info");

    env_logger::init_from_env(env);
    let path = args.file;
    let fs = TarFs::new(path);
    fuser::mount2(fs, args.mount, &[MountOption::AllowOther]).unwrap();
}
