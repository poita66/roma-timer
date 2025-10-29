//! Build script for Roma Timer
//! Handles embedding frontend assets and database migrations

use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=frontend/web-build");
    println!("cargo:rerun-if-changed=migrations");

    // Generate build information
    #[cfg(feature = "build-info")]
    {
        vergen::EmitBuilder::builder()
            .build_timestamp()
            .emit()
            .unwrap();
    }

    // Embed frontend assets if they exist
    let frontend_dist = Path::new("frontend/web-build");
    if frontend_dist.exists() {
        println!("cargo:warning=Embedding frontend assets from {}", frontend_dist.display());

        // Use include_dir to embed frontend assets
        println!("cargo:rustc-cfg=feature=\"embedded-frontend\"");
    }

    // Set up code generation for database migrations if needed
    // This is useful for including migrations in the binary
    let migrations_dir = Path::new("migrations");
    if migrations_dir.exists() {
        println!("cargo:rerun-if-changed=migrations");
    }
}