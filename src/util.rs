use std::path::PathBuf;

pub fn expand_path(path: impl AsRef<str>) -> PathBuf {
    // TODO: Do i need to specialize this to just linux/unix/bsd?
    let path = shellexpand::tilde(path.as_ref());
    PathBuf::from(path.as_ref())
}
