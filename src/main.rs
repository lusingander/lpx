mod app;
mod config;
mod event;
mod image;
mod macros;
mod player;
mod protocol;

use clap::{Parser, ValueEnum};

/// LPX - Terminal Animated GIF Viewer 📽️
#[derive(Parser)]
#[command(version)]
struct Args {
    /// Path to the image file
    file: String,

    /// Select the graphics protocol [default: auto]
    #[arg(short, long, value_name = "TYPE")]
    protocol: Option<ImageProtocolType>,

    /// Number of frames to skip per step action
    #[arg(short = 'n', long, value_name = "N", default_value_t = 10)]
    frame_step: usize,

    /// Limit the maximum width of the UI
    #[arg(short = 'w', long, value_name = "WIDTH")]
    max_width: Option<u16>,

    /// Enable inline mode with the specified height in rows
    #[arg(short, long, name = "HEIGHT", verbatim_doc_comment)]
    inline: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ImageProtocolType {
    Auto,
    Iterm,
    Kitty,
}

impl From<Option<ImageProtocolType>> for protocol::ImageProtocol {
    fn from(protocol: Option<ImageProtocolType>) -> Self {
        match protocol {
            Some(ImageProtocolType::Auto) => protocol::auto_detect(),
            Some(ImageProtocolType::Iterm) => protocol::ImageProtocol::Iterm2,
            Some(ImageProtocolType::Kitty) => protocol::ImageProtocol::Kitty,
            None => protocol::auto_detect(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = config::Config::default();

    let protocol = args.protocol.into();
    let frame_step = args.frame_step;
    let max_width = args.max_width;

    let file = std::fs::File::open(&args.file)?;
    let images = image::Images::load(file, &args.file)?;

    initialize_panic_handler(args.inline);
    let mut terminal = setup(args.inline)?;

    let (tx, rx) = event::new();
    let mut app = app::App::new(images, protocol, frame_step, max_width, config.theme, tx);

    let ret = app.start(&mut terminal, rx);

    shutdown(args.inline)?;
    ret
}

fn setup(inline: Option<u16>) -> std::io::Result<ratatui::DefaultTerminal> {
    ratatui::crossterm::terminal::enable_raw_mode()?;
    if inline.is_none() {
        ratatui::crossterm::execute!(
            std::io::stdout(),
            ratatui::crossterm::terminal::EnterAlternateScreen
        )?;
    }

    let backend = ratatui::prelude::CrosstermBackend::new(std::io::stdout());
    let viewport = if let Some(height) = inline {
        ratatui::Viewport::Inline(height)
    } else {
        ratatui::Viewport::Fullscreen
    };
    ratatui::Terminal::with_options(backend, ratatui::TerminalOptions { viewport })
}

fn shutdown(inline: Option<u16>) -> std::io::Result<()> {
    if inline.is_none() {
        ratatui::crossterm::execute!(
            std::io::stdout(),
            ratatui::crossterm::terminal::LeaveAlternateScreen
        )?;
    }
    ratatui::crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn initialize_panic_handler(inline: Option<u16>) {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        shutdown(inline).unwrap();
        original_hook(panic_info);
    }));
}
