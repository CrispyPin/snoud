use std::io;

#[cfg(target_os = "windows")]
use winres::WindowsResource;

fn main() -> io::Result<()> {
	#[cfg(target_os = "windows")]
	{
		WindowsResource::new().set_icon("icon.ico").compile()
	}
	#[cfg(not(target_os = "windows"))]
	{
		Ok(())
	}
}
