fn main() {
    println!("cargo:rerun-if-changed=static/style.scss");

    let scss = std::fs::read_to_string("static/style.scss")
        .expect("failed to read static/style.scss");

    let css = grass::from_string(scss, &grass::Options::default())
        .expect("failed to compile SCSS");

    let out_dir = std::env::var("OUT_DIR")
        .expect("OUT_DIR not set");

    std::fs::write(format!("{}/style.css", out_dir), &css)
        .expect("failed to write compiled CSS");
}
