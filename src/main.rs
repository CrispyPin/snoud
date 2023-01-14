use rodio::source::{Repeat, Source};
use rodio::{Decoder, OutputStream};
use std::fs::File;
use std::time::Duration;

type Volume = f32;

fn main() {
	let files: Vec<String> = vec![
		"sound/rain.mp3".into(),
		"sound/thunder.mp3".into(),
		"sound/wind.mp3".into(),
	];
	let source = Snoud::new(&files);

	let (_stream, stream_handle) = OutputStream::try_default().unwrap();

	stream_handle.play_raw(source.convert_samples()).unwrap();

	// let sink = Sink::try_new(&stream_handle).unwrap();
	// sink.append(source);
	// sink.play();
	loop {
		std::thread::sleep(Duration::from_millis(200));
	}
}

struct Snoud {
	channels: Vec<SoundChannel>,
	sample_rate: u32,
}

struct SoundChannel {
	source: Repeat<Decoder<File>>,
	paused: bool,
	volume: Volume,
}

impl SoundChannel {
	fn new(name: &String) -> Self {
		let file = File::open(name).expect("File not found");
		let source = Decoder::new(file)
			.expect("Could not decode file")
			.repeat_infinite();
		Self {
			source,
			paused: false,
			volume: 1.0,
		}
	}
}

impl Iterator for Snoud {
	type Item = i16;

	fn next(&mut self) -> Option<Self::Item> {
		let mut out: Self::Item = 0;
		for c in &mut self.channels {
			if c.paused {
				continue;
			}
			let mut sample: Self::Item = c.source.next().unwrap();
			sample = (f32::from(sample) * c.volume) as Self::Item;
			out = out.saturating_add(sample);
		}
		Some(out)
	}
}

impl Source for Snoud {
	fn channels(&self) -> u16 {
		2
	}

	fn sample_rate(&self) -> u32 {
		self.sample_rate
	}

	fn current_frame_len(&self) -> Option<usize> {
		None
	}

	fn total_duration(&self) -> Option<Duration> {
		None
	}
}

impl Snoud {
	fn new(filenames: &[String]) -> Self {
		let channels = filenames.iter().map(SoundChannel::new).collect();

		Self {
			sample_rate: 48000,
			channels,
		}
	}
}
