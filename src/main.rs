use crossterm::cursor::{self, MoveTo};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::ExecutableCommand;
use rodio::Sink;
use rodio::{OutputStream, OutputStreamHandle};
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
	volume_sync: Arc<Mutex<f32>>,
	muted: bool,
}

impl UIChannel {
	fn mute(&mut self) {
		if self.muted {
			self.sync();
		} else {
			*self.volume_sync.lock().unwrap() = 0.0;
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
		*self.volume_sync.lock().unwrap() = self.get_vol();
	}
}

struct App {
	channels: Vec<UIChannel>,
	selected: usize,
	_stream: (OutputStream, OutputStreamHandle),
	sink: Sink,
	volume: i32,
	playing: bool,
	quit: bool,
}

impl App {
	fn new() -> Self {
		let (stream, stream_handle) = OutputStream::try_default() //
			.expect("Failed to create output stream");
		let sink = Sink::try_new(&stream_handle).unwrap();

		Self {
			channels: Vec::new(),
			selected: 0,
			_stream: (stream, stream_handle),
			sink,
			playing: true,
			volume: 20,
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
				volume_sync: internal_volume,
				muted: false,
			};
			self.channels.push(ui_channel);
		}

		self.sink.append(snoud);
		self.sink.play();
		self.sync_vol();

		terminal::enable_raw_mode().unwrap();
		stdout().execute(Clear(ClearType::All)).unwrap();
		stdout().execute(cursor::Hide).unwrap();

		while !self.quit {
			self.render();
			self.input();
		}
		stdout().execute(cursor::Show).unwrap();
		terminal::disable_raw_mode().unwrap();
		println!("Exiting");
	}

	fn render(&mut self) {
		stdout().execute(MoveTo(0, 0)).unwrap();

		println!("Snoud - ambient sound player\r");
		println!(
			"Master volume: {:3}% {:10}\n\r",
			self.volume,
			if self.playing {
				"[Playing]"
			} else {
				"[Paused]"
			}
		);
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
		println!("Play/pause: Space\r");
		println!("Master volume: [.] [,]\r");
		println!("Select channel: Up/Down\r");
		println!("Mute channel: M\r");
		println!("Channel volume: Left/Right\r");
	}

	fn input(&mut self) {
		if !event::poll(Duration::from_millis(50)).unwrap() {
			return;
		}

		let Ok(Event::Key(event)) = event::read() else { return };

		match event.code {
			KeyCode::Char('q') => self.quit = true,
			KeyCode::Up => self.select_prev(),
			KeyCode::Down => self.select_next(),
			KeyCode::Right => self.channels[self.selected].change_vol(10),
			KeyCode::Left => self.channels[self.selected].change_vol(-10),
			KeyCode::Char('m') => self.channels[self.selected].mute(),
			KeyCode::Char(' ') => self.mute(),
			KeyCode::Char('.') => self.inc_vol(),
			KeyCode::Char(',') => self.dec_vol(),
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

	fn inc_vol(&mut self) {
		if self.volume < 15 {
			self.volume += 1;
		} else {
			self.volume += 5;
		}
		self.sync_vol();
	}

	fn dec_vol(&mut self) {
		if self.volume <= 15 {
			self.volume -= 1;
		} else {
			self.volume -= 5;
		}
		self.sync_vol();
	}

	fn sync_vol(&mut self) {
		self.volume = self.volume.clamp(0, 150);
		self.sink.set_volume(self.volume as f32 / 100.0);
	}

	fn mute(&mut self) {
		self.playing = !self.playing;
		if self.playing {
			self.sink.play();
		} else {
			self.sink.pause();
		}
	}
}
