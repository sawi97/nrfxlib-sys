//! Build Script for nrfxlib-sys
//!
//! Calls out to bindgen to generate a Rust crate from the Nordic header
//! files.

fn main() {
    use std::env;
    use std::path::{Path, PathBuf};
    let nrfxlib_path = "./third_party/nordic/nrfxlib";

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .parse_callbacks(Box::new(DocCallback))
        // Point to Nordic headers
        .clang_arg(format!("-I{}", nrfxlib_path))
        // Point to our special local headers
        .clang_arg("-I./include")
        .clang_arg("-I./third_party/nordic/nrfxlib/crypto/nrf_cc310_platform/include")
        .clang_arg("-I./third_party/nordic/nrfxlib/crypto/nrf_oberon")
        // Disable standard includes (they belong to the host)
        .clang_arg("-nostdinc")
        .use_core()
        .ctypes_prefix("core::ffi")
        // Include only the useful stuff
        .allowlist_function("nrf_.*")
        .allowlist_type("nrf_.*")
        .allowlist_var("NRF_.*")
        // Some macros from nrf_modem.h are prefixed with MODEM_
        .allowlist_var("MODEM_.*")
        // Format the output
        .rustfmt_bindings(true)
        // Use signed macro const type
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");

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

#[derive(Debug)]
struct DocCallback;

impl bindgen::callbacks::ParseCallbacks for DocCallback {
    fn process_comment(&self, comment: &str) -> Option<String> {
        let mut comment = comment.to_owned();
        // Format inline @brief
        let re = regex::Regex::new("\\s*@brief\\s*(?P<msg>.*)").unwrap();
        comment = re.replace_all(&comment, "$msg").into();

        // Format deprecation notice (@deprecated) as deprecated
        let re = regex::Regex::new(r"@deprecated").unwrap();
        comment = re
            .replace_all(&comment, "**Deprecated**")
            .into();

        // Format @param as list element
        let re = regex::Regex::new(r"\s*@[pP]aram\s*(\[(?P<typ>[\w,\s]+)\s*\])?\s*(\\t)?(?P<var>[\w\.]+)\s+").unwrap();
        comment = re.replace_all(&comment, "\n * `$var` $typ - ").into();

        // Format @details as a section
        let re = regex::Regex::new(r"\s*@details?\s*(?P<var>.*)").unwrap();
        comment = re.replace_all(&comment, "\n# Details \n$var").into();

        // Format inline @note as bold
        let re = regex::Regex::new(r"\s*@note:?\s*").unwrap();
        comment = re.replace_all(&comment, "\n\n**Note:** ").into();

        // Format references
        // $ref, <post> in case of newlines
        let re = regex::Regex::new(r"\s*@(ref|refitem)\s+(?P<var>\w+)(\(\))?\.?(?P<post>\s+)").unwrap();
        comment = re.replace_all(&comment, " [$var]$post").into();

        // NRF_*
        let re = regex::Regex::new(r"(?P<pre>\s*)(@[pac])?[\s-]+#?(?P<var>NRF_\w+)(?P<post>\s*)").unwrap();
        comment = re.replace_all(&comment, "$pre[$var]$post").into();

        // Return values (@retval/@returns/@return)
        let re = regex::Regex::new(r"@(returns?|retval)\s+(?P<min>\-?)\s?(?P<var>\[?\w+\]?)").unwrap();

        // First entry is header, rest are list items
        comment = re.replace(&comment, "\n# Returns\n * $var ").into();
        comment = re.replace_all(&comment, "\n * $min$var ").into();

        // Format @p/@a/@c arguments as inline code
        let re = regex::Regex::new(r"@[pac]\s+(?P<var>[\*A-Za-z0-9_\(\)]+)").unwrap();
        comment = re.replace_all(&comment, " `$var` ").into();

        Some(comment)
    }
}
