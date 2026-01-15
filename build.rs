fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set("FileDescription", "GPUI Clipboard Manager");
        res.set("Subsystem", "windows");
        res.compile().unwrap();
    }
}
