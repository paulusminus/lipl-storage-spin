use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        wasm: { target_family = "wasm" },
        not_wasm: { not(target_family = "wasm") }
    }
}