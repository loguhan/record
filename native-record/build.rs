fn main() {
    println!("cargo:rerun-if-changed=../src-tauri/icons/icon.ico");

    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("../src-tauri/icons/icon.ico");
        resource.set("FileDescription", "Record");
        resource.set("ProductName", "Record");
        resource.set("OriginalFilename", "record-native.exe");

        if let Err(error) = resource.compile() {
            println!("cargo:warning=failed to embed Windows icon: {error}");
        }
    }
}
