use std::io;


fn main() -> io::Result<()> {
    // Initialize Windows resource builder
    let mut res = winres::WindowsResource::new();

    // res.set_icon("icon.ico");
    res.set_language(winapi::winnt::MAKELANGID(
        winapi::winnt::LANG_ENGLISH,
        winapi::winnt::SUBLANG_ENGLISH_US
    ));
    // Compile resources
    res.compile()?;
    Ok(())
}