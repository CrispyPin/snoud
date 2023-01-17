use rodio::source::{Repeat, Source};
use rodio::Decoder;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct Snoud {
	channels: Vec<SoundChannel>,
	sample_rate: u32,
}

struct SoundChannel {
	source: Repeat<Decoder<File>>,
	paused: bool,
	volume: f32,
	volume_sync: Arc<Mutex<f32>>,
}

impl SoundChannel {
	fn new(name: &str) -> Self {
		let file = File::open(name).expect("File not found");
		let source = Decoder::new(file)
			.expect("Could not decode file")
			.repeat_infinite();
		Self {
			source,
			paused: false,
			volume: 1.0,
			volume_sync: Arc::new(Mutex::new(1.0)),
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
	pub fn new(/* filenames: &[String] */) -> Self {
		// let channels = filenames.iter().map(SoundChannel::new).collect();
		Self {
			sample_rate: 48000,
			channels: Vec::new(),
		}
	}

	pub fn add_channel(&mut self, filename: &str) -> Arc<Mutex<f32>> {
		let new = SoundChannel::new(filename);
		let volume_sync = new.volume_sync.clone();
		self.channels.push(new);
		volume_sync
	}
}
