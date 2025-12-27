fn main() {
    // Compile Slint UI at build-time and bundle translations from `lang/`.
    let config = slint_build::CompilerConfiguration::new()
        .with_bundled_translations("lang")
        .with_style("native".to_string());

    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
