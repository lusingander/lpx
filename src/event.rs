use std::{sync::mpsc, thread};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum AppEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    NextFrame,
}

pub fn new() -> (mpsc::Sender<AppEvent>, mpsc::Receiver<AppEvent>) {
    let (tx, rx) = mpsc::channel();

    let event_tx = tx.clone();
    thread::spawn(move || {
        loop {
            match ratatui::crossterm::event::read().unwrap() {
                ratatui::crossterm::event::Event::Key(ev) => {
                    event_tx.send(AppEvent::Key(ev)).unwrap();
                }
                ratatui::crossterm::event::Event::Resize(w, h) => {
                    event_tx.send(AppEvent::Resize(w, h)).unwrap();
                }
                _ => {}
            };
        }
    });

    (tx, rx)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserEvent {
    Quit,
    SelectNextFrame,
    SelectPrevFrame,
    SelectNextFrameStep,
    SelectPrevFrameStep,
    SelectFirstFrame,
    SelectLastFrame,
    SelectPercentageFrame(u8),
    SelectNextSpeed,
    SelectPrevSpeed,
    TogglePlaying,
    ToggleDetail,
    SetLoopStart,
    SetLoopEnd,
    ClearLoop,
}

pub struct UserEventMapper {
    map: Vec<(KeyEvent, UserEvent)>,
}

impl UserEventMapper {
    pub fn new() -> Self {
        #[rustfmt::skip]
        let map = vec![
            (KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL), UserEvent::Quit),
            (KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE), UserEvent::Quit),
            (KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), UserEvent::Quit),
            (KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE), UserEvent::SelectNextFrame),
            (KeyEvent::new(KeyCode::Right, KeyModifiers::NONE), UserEvent::SelectNextFrame),
            (KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE), UserEvent::SelectPrevFrame),
            (KeyEvent::new(KeyCode::Left, KeyModifiers::NONE), UserEvent::SelectPrevFrame),
            (KeyEvent::new(KeyCode::Char('L'), KeyModifiers::NONE), UserEvent::SelectNextFrameStep),
            (KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT), UserEvent::SelectNextFrameStep),
            (KeyEvent::new(KeyCode::Char('H'), KeyModifiers::NONE), UserEvent::SelectPrevFrameStep),
            (KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT), UserEvent::SelectPrevFrameStep),
            (KeyEvent::new(KeyCode::Char('^'), KeyModifiers::NONE), UserEvent::SelectFirstFrame),
            (KeyEvent::new(KeyCode::Char('$'), KeyModifiers::NONE), UserEvent::SelectLastFrame),
            (KeyEvent::new(KeyCode::Char('0'), KeyModifiers::NONE), UserEvent::SelectPercentageFrame(0)),
            (KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE), UserEvent::SelectPercentageFrame(1)),
            (KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE), UserEvent::SelectPercentageFrame(2)),
            (KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE), UserEvent::SelectPercentageFrame(3)),
            (KeyEvent::new(KeyCode::Char('4'), KeyModifiers::NONE), UserEvent::SelectPercentageFrame(4)),
            (KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE), UserEvent::SelectPercentageFrame(5)),
            (KeyEvent::new(KeyCode::Char('6'), KeyModifiers::NONE), UserEvent::SelectPercentageFrame(6)),
            (KeyEvent::new(KeyCode::Char('7'), KeyModifiers::NONE), UserEvent::SelectPercentageFrame(7)),
            (KeyEvent::new(KeyCode::Char('8'), KeyModifiers::NONE), UserEvent::SelectPercentageFrame(8)),
            (KeyEvent::new(KeyCode::Char('9'), KeyModifiers::NONE), UserEvent::SelectPercentageFrame(9)),
            (KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE), UserEvent::SelectNextSpeed),
            (KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), UserEvent::SelectNextSpeed),
            (KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE), UserEvent::SelectPrevSpeed),
            (KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), UserEvent::SelectPrevSpeed),
            (KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE), UserEvent::TogglePlaying),
            (KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE), UserEvent::ToggleDetail),
            (KeyEvent::new(KeyCode::Char('['), KeyModifiers::NONE), UserEvent::SetLoopStart),
            (KeyEvent::new(KeyCode::Char(']'), KeyModifiers::NONE), UserEvent::SetLoopEnd),
            (KeyEvent::new(KeyCode::Char('C'), KeyModifiers::NONE), UserEvent::ClearLoop),
        ];
        Self { map }
    }

    pub fn find_events(&self, e: KeyEvent) -> Vec<UserEvent> {
        self.map
            .iter()
            .filter_map(|(k, v)| if *k == e { Some(*v) } else { None })
            .collect()
    }
}
