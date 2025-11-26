fn main() -> () {
    let config: slint_build::CompilerConfiguration = slint_build::CompilerConfiguration::new().with_bundled_translations("lang").with_style("native".to_string());

    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
