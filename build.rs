fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");

    res.set("FileDescription", "Velo Command Palette");
    res.set("ProductName", "Velo");
    res.set("FileVersion", "0.1.0");
    res.set("ProductVersion", "0.1.0");

    res.compile().unwrap();
}
