use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::ExecutableCommand;
use rodio::{source::Source, OutputStream, OutputStreamHandle};
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
	volume: f32,
	internal_volume: Arc<Mutex<f32>>,
}

struct App {
	channels: Vec<UIChannel>,
	selected_index: usize,
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
			selected_index: 0,
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
		stdout().execute(Clear(ClearType::All)).unwrap();

		while !self.quit {
			self.render();
			self.input();
		}
		terminal::disable_raw_mode().unwrap();
	}

	fn render(&mut self) {
		stdout().execute(MoveTo(0, 0)).unwrap();

		println!("Snoud - ambient sound player\n\r");
		for (i, channel) in self.channels.iter().enumerate() {
			println!(
				"{}{}:\r\n {:3.0}%\r\n",
				if i == self.selected_index { ">" } else { " " },
				&channel.name,
				(channel.volume * 100.0)
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
			KeyCode::Right => self.set_channel_volume(0.1),
			KeyCode::Left => self.set_channel_volume(-0.1),
			_ => (),
		}
	}

	fn set_channel_volume(&mut self, delta: f32) {
		let channel = self.channels.get_mut(self.selected_index).unwrap();
		channel.volume = (channel.volume + delta).clamp(0., 2.);
		*channel.internal_volume.lock().unwrap() = channel.volume;
	}

	fn select_prev(&mut self) {
		self.selected_index = match self.selected_index {
			0 => self.channels.len() - 1,
			n => n - 1,
		};
	}

	fn select_next(&mut self) {
		self.selected_index = (self.selected_index + 1) % self.channels.len();
	}
}
