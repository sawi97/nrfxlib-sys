//! Build Script for nrfxlib-sys
//!
//! Calls out to bindgen to generate a Rust crate from the Nordic header
//! files.

fn main() {
    use std::env;
    use std::path::{Path, PathBuf};
    let nrfxlib_path = "./third_party/nordic/nrfxlib";

    // Generate bindings with bindgen
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .parse_callbacks(Box::new(DocCallback))
        // Point to Nordic headers
        .clang_arg("-I./include")
        .clang_arg(format!("-I{}", nrfxlib_path))
        .clang_arg(format!("-I{}/crypto/nrf_cc310_platform/include", nrfxlib_path))
        .clang_arg(format!("-I{}/crypto/nrf_oberon", nrfxlib_path))
        // Disable standard includes (they belong to the host)
        .clang_arg("-nostdinc")
        .use_core()
        .ctypes_prefix("core::ffi")
        // Include only the useful stuff
        .allowlist_function("nrf_.*")
        .allowlist_type("nrf_.*")
        .allowlist_var("NRF_.*")
        .allowlist_function("ocrypto_.*")
        // Format the output
        .formatter(bindgen::Formatter::Prettyplease)
        // Use signed macro const type
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");

    // Evaluate which library to use, based on the log feature
    let modem_lib: String = if cfg!(feature = "log") {
        "modem_log".into()
    } else {
        "modem".into()
    };
    let modem_libf = format!("lib{}.a", modem_lib);
    let sip = "nRF9160";

    // Copy libraries to the output directory (makes it easier to detect changed library names on future updates)
    let libmodem_path = Path::new(&nrfxlib_path)
        .join(format!("nrf_modem/lib/{sip}/hard-float/{modem_libf}"));
    let liboberon_path = Path::new(&nrfxlib_path)
        .join("crypto/nrf_oberon/lib/cortex-m33/hard-float/liboberon_3.0.13.a");
    let libcc310_path = Path::new(&nrfxlib_path)
        .join("crypto/nrf_cc310_platform/lib/cortex-m33/hard-float/no-interrupts/libnrf_cc310_platform_0.9.18.a");

    std::fs::copy(libmodem_path.clone(), out_path.join(libmodem_path.file_name().unwrap())).unwrap();
    std::fs::copy(liboberon_path.clone(), out_path.join(liboberon_path.file_name().unwrap())).unwrap();
    std::fs::copy(libcc310_path.clone(), out_path.join(libcc310_path.file_name().unwrap())).unwrap();

    // Link libraries
    println!("cargo:rustc-link-search={}", out_path.display());

    println!("cargo:rustc-link-lib=static={}", modem_lib);
    println!("cargo:rustc-link-lib=static=nrf_cc310_platform_0.9.18");
    println!("cargo:rustc-link-lib=static=oberon_3.0.13");
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
