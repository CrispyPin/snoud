use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::ExecutableCommand;
use rodio::{source::Source, OutputStream, OutputStreamHandle};
use std::fs;
use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod sound;
use sound::Snoud;

fn main() {
	App::new().run();
}

struct UIChannel {
	name: String,
	volume: i32,
	internal_volume: Arc<Mutex<f32>>,
	muted: bool,
}

impl UIChannel {
	fn mute(&mut self) {
		if self.muted {
			self.sync();
		} else {
			*self.internal_volume.lock().unwrap() = 0.0;
		}
		self.muted = !self.muted;
	}

	fn get_vol(&self) -> f32 {
		self.volume as f32 / 100.0
	}

	fn change_vol(&mut self, amt: i32) {
		self.muted = false;
		self.volume = (self.volume + amt).clamp(0, 200);
		self.sync();
	}

	fn sync(&mut self) {
		*self.internal_volume.lock().unwrap() = self.get_vol();
	}
}

struct App {
	channels: Vec<UIChannel>,
	selected: usize,
	_stream: OutputStream,
	stream_handle: OutputStreamHandle,
	quit: bool,
}

impl App {
	fn new() -> Self {
		let (_stream, stream_handle) = OutputStream::try_default() //
			.expect("Failed to create output stream");
		Self {
			channels: Vec::new(),
			selected: 0,
			_stream,
			stream_handle,
			quit: false,
		}
	}

	fn run(&mut self) {
		let files: Vec<_> = fs::read_dir("sound")
			.unwrap()
			.flatten()
			.filter(|f| f.file_type().unwrap().is_file())
			.collect();

		let mut snoud = Snoud::new();
		for file in files {
			let internal_volume = snoud.add_channel(&file.path());
			let ui_channel = UIChannel {
				name: file.file_name().to_string_lossy().into(),
				volume: 100,
				internal_volume,
				muted: false,
			};
			self.channels.push(ui_channel);
		}

		self.stream_handle
			.play_raw(snoud.convert_samples())
			.unwrap();

		terminal::enable_raw_mode().unwrap();
		stdout().execute(Clear(ClearType::All)).unwrap();

		while !self.quit {
			self.render();
			self.input();
		}
		terminal::disable_raw_mode().unwrap();
		println!("Exiting");
	}

	fn render(&mut self) {
		stdout().execute(MoveTo(0, 0)).unwrap();

		println!("Snoud - ambient sound player\n\r");
		for (i, channel) in self.channels.iter().enumerate() {
			println!(
				"{selection} {name}:\r\n  {volume:3.0}% {status:-<21}\r\n",
				selection = if i == self.selected { ">" } else { " " },
				name = &channel.name,
				volume = channel.volume,
				status = if channel.muted {
					"[Muted]".to_owned()
				} else {
					format!(
						"{:=>width$}",
						"|",
						width = (channel.volume / 10 + 1) as usize
					)
				}
			);
		}
	}

	fn input(&mut self) {
		if !event::poll(Duration::from_millis(50)).unwrap() {
			return;
		}

		let event = if let Ok(Event::Key(keyevent)) = event::read() {
			keyevent
		} else {
			return;
		};

		match event.code {
			KeyCode::Char('q') => self.quit = true,
			KeyCode::Up => self.select_prev(),
			KeyCode::Down => self.select_next(),
			KeyCode::Right => self.channels[self.selected].change_vol(10),
			KeyCode::Left => self.channels[self.selected].change_vol(-10),
			KeyCode::Char(' ' | 'm') => self.channels[self.selected].mute(),
			_ => (),
		}
	}

	fn select_prev(&mut self) {
		self.selected = match self.selected {
			0 => self.channels.len() - 1,
			n => n - 1,
		};
	}

	fn select_next(&mut self) {
		self.selected = (self.selected + 1) % self.channels.len();
	}
}
