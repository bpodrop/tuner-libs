use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, Stream};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioInputKind {
    Live,
    Mock,
}

#[derive(Debug)]
pub struct AudioStart {
    pub receiver: Receiver<Vec<f32>>,
    pub input_kind: AudioInputKind,
    pub sample_rate_hz: u32,
    pub fallback_reason: Option<String>,
}

#[derive(Debug)]
pub enum AudioError {
    NoInputDevice,
    DefaultInputConfig(cpal::DefaultStreamConfigError),
    BuildStream(cpal::BuildStreamError),
    PlayStream(cpal::PlayStreamError),
    UnsupportedSampleFormat(String),
    AlreadyStarted,
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoInputDevice => write!(f, "no input audio device available"),
            Self::DefaultInputConfig(err) => {
                write!(f, "failed to read default input config: {err}")
            }
            Self::BuildStream(err) => write!(f, "failed to build audio stream: {err}"),
            Self::PlayStream(err) => write!(f, "failed to start audio stream: {err}"),
            Self::UnsupportedSampleFormat(format) => {
                write!(f, "unsupported sample format: {format}")
            }
            Self::AlreadyStarted => write!(f, "audio capture already started"),
        }
    }
}

impl std::error::Error for AudioError {}

pub struct AudioCapture {
    frame_size: usize,
    hop_size: usize,
    sample_rate_hz: u32,
    sender: Option<Sender<Vec<f32>>>,
    stream: Option<Stream>,
    stop_requested: Arc<AtomicBool>,
    mock_thread: Option<JoinHandle<()>>,
}

impl AudioCapture {
    pub fn new(frame_size: usize, hop_size: usize, sample_rate_hz: u32) -> Self {
        Self {
            frame_size,
            hop_size: hop_size.max(1),
            sample_rate_hz: sample_rate_hz.max(1),
            sender: None,
            stream: None,
            stop_requested: Arc::new(AtomicBool::new(false)),
            mock_thread: None,
        }
    }

    pub fn start(&mut self) -> Result<AudioStart, AudioError> {
        if self.sender.is_some() || self.stream.is_some() || self.mock_thread.is_some() {
            return Err(AudioError::AlreadyStarted);
        }

        self.stop_requested.store(false, Ordering::SeqCst);

        let (sender, receiver) = mpsc::channel();
        self.sender = Some(sender.clone());

        match self.try_start_stream(sender.clone()) {
            Ok((stream, sample_rate_hz)) => {
                self.stream = Some(stream);
                Ok(AudioStart {
                    receiver,
                    input_kind: AudioInputKind::Live,
                    sample_rate_hz,
                    fallback_reason: None,
                })
            }
            Err(error) => {
                let fallback_reason = error.to_string();
                self.spawn_mock_source(sender);
                Ok(AudioStart {
                    receiver,
                    input_kind: AudioInputKind::Mock,
                    sample_rate_hz: self.sample_rate_hz,
                    fallback_reason: Some(fallback_reason),
                })
            }
        }
    }

    fn try_start_stream(&self, sender: Sender<Vec<f32>>) -> Result<(Stream, u32), AudioError> {
        let host = cpal::default_host();
        let input_device = host
            .default_input_device()
            .ok_or(AudioError::NoInputDevice)?;
        let input_config = input_device
            .default_input_config()
            .map_err(AudioError::DefaultInputConfig)?;

        let frame_size = self.frame_size;
        let hop_size = self.hop_size;
        let stop_requested = Arc::clone(&self.stop_requested);
        let buffered_samples = Arc::new(Mutex::new(Vec::<f32>::with_capacity(frame_size * 2)));
        let stream_config: cpal::StreamConfig = input_config.clone().into();
        let sample_rate_hz = stream_config.sample_rate.0;
        let err_fn = move |err| eprintln!("Audio stream error: {err}");
        let frame_config = FrameConfig {
            frame_size,
            hop_size,
        };

        let stream = match input_config.sample_format() {
            SampleFormat::F32 => build_stream::<f32>(
                &input_device,
                &stream_config,
                frame_config,
                StreamState {
                    sender,
                    buffered_samples,
                    stop_requested,
                },
                err_fn,
            ),
            SampleFormat::I16 => build_stream::<i16>(
                &input_device,
                &stream_config,
                frame_config,
                StreamState {
                    sender,
                    buffered_samples,
                    stop_requested,
                },
                err_fn,
            ),
            SampleFormat::U16 => build_stream::<u16>(
                &input_device,
                &stream_config,
                frame_config,
                StreamState {
                    sender,
                    buffered_samples,
                    stop_requested,
                },
                err_fn,
            ),
            other => return Err(AudioError::UnsupportedSampleFormat(format!("{other:?}"))),
        }?;

        Ok((stream, sample_rate_hz))
    }

    fn spawn_mock_source(&mut self, sender: Sender<Vec<f32>>) {
        let frame_size = self.frame_size;
        let hop_size = self.hop_size;
        let sample_rate_hz = self.sample_rate_hz as f32;
        let stop_requested = Arc::clone(&self.stop_requested);

        self.mock_thread = Some(thread::spawn(move || {
            let frame = vec![0.0; frame_size];
            while !stop_requested.load(Ordering::SeqCst) {
                if sender.send(frame.clone()).is_err() {
                    break;
                }

                let sleep_ms = ((hop_size as f32 / sample_rate_hz) * 1000.0).max(1.0) as u64;
                thread::sleep(Duration::from_millis(sleep_ms));
            }
        }));
    }

    pub fn stop(&mut self) {
        self.stop_requested.store(true, Ordering::SeqCst);
        self.sender.take();
        self.stream.take();

        if let Some(handle) = self.mock_thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.stop();
    }
}

struct FrameConfig {
    frame_size: usize,
    hop_size: usize,
}

struct StreamState {
    sender: Sender<Vec<f32>>,
    buffered_samples: Arc<Mutex<Vec<f32>>>,
    stop_requested: Arc<AtomicBool>,
}

fn build_stream<T>(
    input_device: &cpal::Device,
    stream_config: &cpal::StreamConfig,
    frame_config: FrameConfig,
    stream_state: StreamState,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, AudioError>
where
    T: Sample + cpal::SizedSample + Send + 'static,
    f32: cpal::FromSample<T>,
{
    let FrameConfig {
        frame_size,
        hop_size,
    } = frame_config;
    let StreamState {
        sender,
        buffered_samples,
        stop_requested,
    } = stream_state;
    let channels = stream_config.channels as usize;
    let stream = input_device
        .build_input_stream(
            stream_config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                if stop_requested.load(Ordering::SeqCst) {
                    return;
                }

                let mut buffer = match buffered_samples.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                for frame in data.chunks(channels) {
                    if let Some(sample) = frame.first() {
                        buffer.push(sample.to_sample::<f32>());
                    }
                }

                for chunk in extract_sliding_frames(&mut buffer, frame_size, hop_size) {
                    if sender.send(chunk).is_err() {
                        return;
                    }
                }
            },
            err_fn,
            None,
        )
        .map_err(AudioError::BuildStream)?;

    stream.play().map_err(AudioError::PlayStream)?;
    Ok(stream)
}

fn extract_sliding_frames(
    buffered_samples: &mut Vec<f32>,
    frame_size: usize,
    hop_size: usize,
) -> Vec<Vec<f32>> {
    if frame_size == 0 {
        buffered_samples.clear();
        return Vec::new();
    }

    let hop_size = hop_size.max(1);
    let mut frames = Vec::new();

    while buffered_samples.len() >= frame_size {
        frames.push(buffered_samples[..frame_size].to_vec());

        let drain_count = hop_size.min(buffered_samples.len());
        buffered_samples.drain(..drain_count);
    }

    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_capture_creation() {
        let audio = AudioCapture::new(4096, 1024, 44_100);
        assert_eq!(audio.frame_size, 4096);
        assert_eq!(audio.hop_size, 1024);
        assert_eq!(audio.sample_rate_hz, 44_100);
    }

    #[test]
    fn stop_is_idempotent_without_starting() {
        let mut audio = AudioCapture::new(1024, 256, 48_000);
        audio.stop();
        audio.stop();
    }

    #[test]
    fn extracts_overlapping_frames_using_hop_size() {
        let mut buffer = (0..8).map(|sample| sample as f32).collect::<Vec<_>>();

        let frames = extract_sliding_frames(&mut buffer, 4, 2);

        assert_eq!(
            frames,
            vec![
                vec![0.0, 1.0, 2.0, 3.0],
                vec![2.0, 3.0, 4.0, 5.0],
                vec![4.0, 5.0, 6.0, 7.0],
            ]
        );
        assert_eq!(buffer, vec![6.0, 7.0]);
    }

    #[test]
    fn normalizes_zero_hop_size_to_one_sample() {
        let mut buffer = vec![0.0, 1.0, 2.0, 3.0];

        let frames = extract_sliding_frames(&mut buffer, 4, 0);

        assert_eq!(frames, vec![vec![0.0, 1.0, 2.0, 3.0]]);
        assert_eq!(buffer, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn audio_start_reports_mock_input_kind() {
        let start = AudioStart {
            receiver: mpsc::channel().1,
            input_kind: AudioInputKind::Mock,
            sample_rate_hz: 44_100,
            fallback_reason: Some("no input audio device available".to_string()),
        };

        assert_eq!(start.input_kind, AudioInputKind::Mock);
        assert_eq!(start.sample_rate_hz, 44_100);
        assert_eq!(
            start.fallback_reason.as_deref(),
            Some("no input audio device available")
        );
    }
}
