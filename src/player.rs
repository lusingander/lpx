use std::{
    sync::{Arc, Condvar, Mutex, mpsc},
    thread,
    time::Duration,
};

use crate::event::AppEvent;

struct PlayerState {
    playing: bool,
    delay_ms: u32,
}

pub struct Player {
    state: Arc<(Mutex<PlayerState>, Condvar)>,
}

impl Player {
    pub fn new(tx: mpsc::Sender<AppEvent>, delay_ms: u32) -> Self {
        let state = Arc::new((
            Mutex::new(PlayerState {
                playing: false,
                delay_ms,
            }),
            Condvar::new(),
        ));

        let _state = state.clone();
        thread::spawn(move || {
            let (lock, cvar) = &*_state;
            loop {
                let mut state = lock.lock().unwrap();
                while !state.playing {
                    // Wait until playing becomes true
                    state = cvar.wait(state).unwrap();
                    continue;
                }

                let delay = Duration::from_millis(state.delay_ms as u64);
                let (new_state, result) = cvar.wait_timeout(state, delay).unwrap();
                state = new_state;

                // If timed out and still playing, trigger next frame
                // If not timed out, it means something (playing or delay) changed
                if result.timed_out() && state.playing {
                    tx.send(AppEvent::NextFrame).unwrap();
                }
            }
        });

        Self { state }
    }

    pub fn play(&self) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap();
        state.playing = true;
        cvar.notify_all();
    }

    pub fn pause(&self) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap();
        state.playing = false;
        cvar.notify_all();
    }

    pub fn is_playing(&self) -> bool {
        let (lock, _) = &*self.state;
        lock.lock().unwrap().playing
    }

    pub fn set_delay_ms(&self, delay_ms: u32) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap();
        if state.delay_ms != delay_ms {
            state.delay_ms = delay_ms;
            cvar.notify_all();
        }
    }
}
