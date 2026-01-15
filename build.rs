fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set("FileDescription", "GPUI Clipboard Manager");
        res.set_subsystem(winres::WindowsSubsystem::Windows);
        res.compile().unwrap();
    }
}
