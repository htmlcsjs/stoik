use std::{env, fs, path::Path};

use image::EncodableLayout;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("icon.rs");
    let img = image::open(Path::new("assets/icon-512.png"))
        .unwrap()
        .to_rgba8();
    let bytes = img.as_bytes();

    fs::write(
        dest_path,
        format!(
            "
pub const ICON_BYTES: [u8; {}] = {bytes:?};
pub const ICON_SIZE: (u32, u32) = {:?};",
            bytes.len(),
            img.dimensions()
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=assets/icon.png");
}
