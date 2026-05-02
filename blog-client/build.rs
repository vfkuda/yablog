use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/blog.proto");

    let descriptor_path = PathBuf::from(env::var("OUT_DIR").expect("expect OUT_DIR in build.rs"))
        .join("blog_srv_descriptor.bin");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&["proto/blog.proto"], &["proto"])?;
    Ok(())
}
