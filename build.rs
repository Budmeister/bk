
fn main() {
    cc::Build::new()
    .file("src/boot.s")
    .compile("bootstrap");
}