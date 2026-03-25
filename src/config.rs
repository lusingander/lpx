use ratatui::style::Color;
use smart_default::SmartDefault;

#[derive(Default)]
pub struct Config {
    pub theme: Theme,
}

#[derive(SmartDefault)]
pub struct Theme {
    #[default(Color::Green)]
    pub gauge_filled_fg: Color,
    #[default(Color::DarkGray)]
    pub gauge_unfilled_fg: Color,
    #[default(Color::Reset)]
    pub gauge_loop_marker_fg: Color,
    #[default(Color::Reset)]
    pub file_fg: Color,
    #[default(Color::Reset)]
    pub loop_fg: Color,
    #[default(Color::Reset)]
    pub speed_fg: Color,
    #[default(Color::Reset)]
    pub frame_fg: Color,
    #[default(Color::Reset)]
    pub detail_fg: Color,
    #[default(Color::DarkGray)]
    pub divider_fg: Color,
}
