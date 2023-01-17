use crossterm::{
	cursor::MoveTo,
	event::{self, Event, KeyCode},
	execute,
	terminal::{self, Clear, ClearType},
};
use rodio::{source::Source, OutputStream, OutputStreamHandle};
use std::{
	io::stdout,
	sync::{Arc, Mutex},
	time::Duration,
};

mod sound;
use sound::Snoud;

fn main() {
	App::new().run();
}

struct UIChannel {
	name: String,
	volume: f32,
	internal_volume: Arc<Mutex<f32>>,
}

struct App {
	channels: Vec<UIChannel>,
	selected_channel: usize,
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
			selected_channel: 0,
			_stream,
			stream_handle,
			quit: false,
		}
	}

	fn run(&mut self) {
		// TODO scan directory instead
		let files: Vec<String> = vec![
			"sound/rain.mp3".into(),
			"sound/thunder.mp3".into(),
			"sound/wind.mp3".into(),
		];

		let mut snoud = Snoud::new();
		for filename in files {
			let internal_volume = snoud.add_channel(&filename);
			let ui_channel = UIChannel {
				name: filename,
				volume: 1.0,
				internal_volume,
			};
			self.channels.push(ui_channel);
		}

		self.stream_handle
			.play_raw(snoud.convert_samples())
			.unwrap();

		terminal::enable_raw_mode().unwrap();
		while !self.quit {
			self.render();
			self.input();
		}
		terminal::disable_raw_mode().unwrap();
	}

	fn render(&mut self) {
		execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();

		terminal::disable_raw_mode().unwrap();
		println!("Snoud!!");
		println!("--------");
		for (i, channel) in self.channels.iter().enumerate() {
			println!(
				"{}{}:\n {:.0}",
				if i == self.selected_channel { ">" } else { " " },
				&channel.name,
				(channel.volume * 100.0)
			);
		}
		terminal::enable_raw_mode().unwrap();
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
			KeyCode::Up => {
				self.selected_channel = match self.selected_channel {
					0 => self.channels.len() - 1,
					n => n - 1,
				};
			}
			KeyCode::Down => {
				self.selected_channel = (self.selected_channel + 1) % self.channels.len();
			}
			_ => (),
		}
	}
}
