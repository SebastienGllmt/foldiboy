fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=../../wit/deps.toml");
    println!("cargo:rerun-if-changed=../../wit/deps.lock");
    println!("cargo:rerun-if-changed=../../interfaces/nightstream-audio");
    println!("cargo:rerun-if-changed=../../wit/deps");
    wit_deps::lock_sync!("../../wit")
}
