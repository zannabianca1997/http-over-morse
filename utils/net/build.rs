use std::env;

fn main() {
    cargo_emit::rerun_if_env_changed!("ADDRESS");
    if env::var_os("ADDRESS").is_none() {
        cargo_emit::warning!(
            "Env var ADDRESS not set: the device will have address 1, collisions might happens"
        )
    }
}
