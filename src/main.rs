use env_logger::Env;
use fuser::MountOption;
use tarfs_rs::TarFs;

fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
    let fs = TarFs::new("test_tar.tar");
    fuser::mount2(fs, "mnt", &[MountOption::AllowOther]).unwrap();
}
