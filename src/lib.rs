mod filesystem;
mod provider;

pub type Provider = provider::Provider<filesystem::RealFileSystem>;


pub fn difftest<F: FnOnce(&mut Provider)>(name: &str, f: F) {
    let mut provider = Provider::new(filesystem::RealFileSystem { root: "f".into() });
    f(&mut provider)
}
