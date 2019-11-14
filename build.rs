fn main() {
    cc::Build::new()
        .file("src/wrapper.c")
        .compile("wrapper");
}
