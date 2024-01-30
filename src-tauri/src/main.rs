// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Instant;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::State;
use fundsp::hacker::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, SampleFormat, SizedSample, StreamConfig};
use fundsp::hacker::{
    hammond_hz, multipass, reverb_stereo, sine, sine_hz, soft_saw_hz, square_hz, wave64, Wave64,
};

use tauri::Manager;

struct Store {
    timer: Mutex<Instant>,
    playing: Mutex<bool>,
}

#[derive(Debug, PartialEq)]
enum Action {
    Stop,
    Play,
    Done,
    Tick,
}

impl FromStr for Action {
    type Err = ();

    fn from_str(input: &str) -> Result<Action, Self::Err> {
        match input {
            "PLAY"  => Ok(Action::Play),
            "STOP"  => Ok(Action::Stop),
            "DONE"  => Ok(Action::Done),
            "TICK" => Ok(Action::Tick),
            _      => Err(()),
        }
    }
}

#[tauri::command]
fn play(playing: bool, playhead: State<Store>) {
    if playing {
        println!("hit play!");
        let now = Instant::now();
        // *playhead.out.lock().unwrap() = wave1;
        *playhead.timer.lock().unwrap() = now;
    } else {
        println!("stopping");
    }

    *playhead.playing.lock().unwrap() = playing;
}

#[tauri::command]
fn current(playhead: State<Store>) -> String {
    let secs = playhead.timer.lock().unwrap().elapsed().as_secs();
    if *playhead.playing.lock().unwrap() {
        format!("it's a sine! {}", secs)
    } else {
        String::from("stopped")
    }
}

fn run_output(audio_graph: Box<dyn AudioUnit64>) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();
    match config.sample_format() {
        SampleFormat::F32 => run_synth::<f32>(audio_graph, device, config.into()),
        SampleFormat::I16 => run_synth::<i16>(audio_graph, device, config.into()),
        SampleFormat::U16 => run_synth::<u16>(audio_graph, device, config.into()),

        _ => panic!("Unsupported format"),
    }
}

fn run_synth<T: SizedSample + FromSample<f64>>(
    mut audio_graph: Box<dyn AudioUnit64>,
    device: Device,
    config: StreamConfig,
) {
    std::thread::spawn(move || {
        let sample_rate = config.sample_rate.0 as f64;
        audio_graph.set_sample_rate(sample_rate);

        // This is a function that is used to get the next audio sample. It is
        // written using the closure syntax, so looks a bit different from
        // normal function definition.
        let mut next_value = move || audio_graph.get_stereo();

        let channels = config.channels as usize;
        let err_fn = |err| eprintln!("an error occurred on stream: {err}");
        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &mut next_value)
                },
                err_fn,
                None,
            )
            .unwrap();

        stream.play().unwrap();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
}

fn create_sine_440() -> Box<dyn AudioUnit64> {
    let synth = sine_hz(440.0);

    Box::new(synth)
}

fn write_data<T: SizedSample + FromSample<f64>>(
    output: &mut [T],
    channels: usize,
    next_sample: &mut dyn FnMut() -> (f64, f64),
) {
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left: T = T::from_sample(sample.0);
        let right: T = T::from_sample(sample.1);

        for (channel, sample) in frame.iter_mut().enumerate() {
            *sample = if channel & 1 == 0 { left } else { right };
        }
    }
}

fn main() {
    let now = Instant::now();
    let audio_graph = create_sine_440();
    run_output(audio_graph);

    tauri::Builder::default()
        .setup(|app| {
            app.listen_global("ACTION", |event| {
                let action = Action::from_str(event.payload().unwrap()).unwrap();
                match action {
                    Action::Play => {
                        println!("play")
                    },
                    Action::Tick => {
                        println!("tick")
                    },
                    Action::Stop => {
                        println!("stop")
                    },
                    Action::Done => {
                        println!("done")
                    },
                }
            });

            Ok(())
        })
        .manage(Store { timer: Mutex::new(now), playing: Mutex::new(false) })
        .invoke_handler(tauri::generate_handler![play, current])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
