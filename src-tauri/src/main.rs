// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Instant;
use std::sync::Mutex;
use tauri::State;
use fundsp::hacker::*;

struct Playhead {
    timer: Mutex<Instant>
    // out: Mutex<Wave64>
}

#[tauri::command]
fn play(playing: bool, playhead: State<Playhead>) {
    if playing {
        println!("hit play!");
        let wave1 = Wave64::render(44100.0, 0.5, &mut (pink()));

        let now = Instant::now();
        // *playhead.out.lock().unwrap() = wave1;
        *playhead.timer.lock().unwrap() = now;
    } else {
        println!("stopping");
    }
}

#[tauri::command]
fn current(playhead: State<Playhead>) -> String {
    let secs = playhead.timer.lock().unwrap().elapsed().as_secs();
    println!("secs {}", secs);
    println!("secs {}", secs);
    format!("{}", secs)
}

fn main() {
    let now = Instant::now();
    tauri::Builder::default()
        .manage(Playhead { timer: Mutex::new(now) })
        .invoke_handler(tauri::generate_handler![play, current])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
