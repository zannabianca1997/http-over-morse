use std::env;

fn main() {
    // Configurations
    cargo_emit::pair!("rustc-check-cfg", "cfg(base32)");
    cargo_emit::rerun_if_env_changed!("BASE32");
    if env::var_os("BASE32").is_some() {
        cargo_emit::rustc_cfg!("base32")
    }
}
