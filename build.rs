//! Build Script for nrfxlib-sys
//!
//! Calls out to bindgen to generate a Rust crate from the Nordic header
//! files.

fn main() {
    use std::env;
    use std::path::{Path, PathBuf};
    let nrfxlib_path = "./third_party/nordic/nrfxlib";
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Point to Nordic headers
        .clang_arg(format!("-I{}", nrfxlib_path))
        // Point to our special local headers
        .clang_arg("-I./include")
        // Add extra paths that the C files assume are searched
        .clang_arg("-I./third_party/nordic/nrfxlib/crypto/nrf_cc310_platform/include")
        .clang_arg("-I./third_party/nordic/nrfxlib/crypto/nrf_oberon")
        // Disable standard includes (they belong to the host)
        .clang_arg("-nostdinc")
        // Set the target
        .clang_arg("-target")
        .clang_arg("arm")
        .clang_arg("-mcpu=cortex-m33")
        // Use hardfp
        .clang_arg("-mfloat-abi=hard")
        // We're no_std
        .use_core()
        // Use our own ctypes to save using libc
        .ctypes_prefix("ctypes")
        // Include only the useful stuff
        .allowlist_function("nrf_.*")
        .allowlist_type("nrf_.*")
        .allowlist_var("NRF_.*")
        // Some macros from nrf_modem.h
        .allowlist_var("MODEM_.*")
        // Format the output
        .rustfmt_bindings(true)
        // Use signed macro const type
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let mut rust_source = bindings.to_string();

    // Munge Doxygen comments into something Rustdoc can handle
    rust_source = rust_source.replace("#[doc = \" @{\"]", "");

    // Format @param as list element
    let re = regex::Regex::new(r"\s*@[pP]aram\s*(\[(?P<typ>[\w,\s]+)\s*\])?\s+(?P<var>[\w\.]+)\s+").unwrap();
    rust_source = re.replace_all(&rust_source, " * `$var` $typ - ").into();

    // Format @p/@a/@c arguments as inline code
    let re = regex::Regex::new(r"@[pac]\s+(?P<var>[A-Za-z0-9_\(\)]+)").unwrap();
    rust_source = re.replace_all(&rust_source, " `$var` ").into();

    // Format NRF_* as ref
    let re = regex::Regex::new(r"(?P<pre>@(returns?|retval|note)\s+.*)(?P<var>NRF_\w+)").unwrap();
    rust_source = re.replace_all(&rust_source, "$pre[$var]").into();

    // #nrf_*
    let re = regex::Regex::new("(?P<pre>#\\[doc.*\\s+)#(?P<var>(nrf|NRF)_\\w+)(?P<post>\\s+.*\"\\])").unwrap();
    rust_source = re
        .replace_all(&rust_source, "$pre[$var]$post")
        .into();

    // Remove @addtogroup stuff
    let re = regex::RegexBuilder::new(r"^#\[doc.*@addtogroup(.|\n)*?^$")
        .multi_line(true)
        .build()
        .unwrap();
    rust_source = re.replace_all(&rust_source, "").into();

    // Format @ref as markdown ref
    let re = regex::Regex::new(r"\s*@(ref|refitem)\s+(?P<var>\w+)").unwrap();
    rust_source = re.replace_all(&rust_source, " [$var]").into();

    // Format deprecation notice as deprecated
    let re = regex::Regex::new("#\\[doc.*@deprecated\\s*(?P<note>.*)\\.*\".*]").unwrap();
    rust_source = re
        .replace_all(&rust_source, "#[deprecated(note=\"$note\")]")
        .into();

    // Format inline markup
    let re = regex::Regex::new(r"\s*@brief\s*(?P<var>.*)").unwrap();
    rust_source = re.replace_all(&rust_source, "$var").into();
    let re = regex::Regex::new(r"\s*@note:?\s+(?P<var>.*)").unwrap();
    rust_source = re.replace_all(&rust_source, "\n\n**Note**: $var").into();

    // Format some sections as headers
    let re = regex::Regex::new(r"\s*@details?\s*(?P<var>.*)").unwrap();
    rust_source = re.replace_all(&rust_source, "# Details \n$var").into();
    rust_source = rust_source.replace("@name ", "# ");
    rust_source = rust_source.replace("@return ", "# Returns\n");
    rust_source = rust_source.replace("@returns ", "# Returns\n");
    rust_source = rust_source.replace("@retval ", "# Returns\n");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    std::fs::write(out_path, rust_source).expect("Couldn't write updated bindgen output");

    // Make sure we link against the libraries
    println!(
        "cargo:rustc-link-search={}",
        Path::new(&nrfxlib_path)
            .join("nrf_modem/lib/cortex-m33/hard-float")
            .display()
    );
    println!(
        "cargo:rustc-link-search={}",
        Path::new(&nrfxlib_path)
            .join("crypto/nrf_oberon/lib/cortex-m33/hard-float")
            .display()
    );
    println!("cargo:rustc-link-lib=static=modem");
    println!("cargo:rustc-link-lib=static=oberon_3.0.12");
}
