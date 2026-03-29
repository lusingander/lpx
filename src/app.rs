use std::sync::mpsc;

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect, Size},
    style::{Style, Stylize},
    text::Line,
    widgets::{LineGauge, Widget},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    config::Theme,
    event::{AppEvent, UserEvent, UserEventMapper},
    handle_user_events,
    image::Images,
    player::Player,
    protocol::ImageProtocol,
};

const DIVIDER: &str = " | ";

pub struct App {
    images: Images,
    current_frame: usize,
    speed_list: Vec<f32>,
    current_speed_index: usize,
    show_details: bool,
    loop_start: Option<usize>,
    loop_end: Option<usize>,
    clear_on_next_render: bool,
    mapper: UserEventMapper,
    player: Player,
    protocol: ImageProtocol,
    frame_step: usize,
    max_width: Option<u16>,
    theme: Theme,
}

impl App {
    pub fn new(
        images: Images,
        protocol: ImageProtocol,
        frame_step: usize,
        max_width: Option<u16>,
        theme: Theme,
        tx: mpsc::Sender<AppEvent>,
    ) -> Self {
        let current_frame = 0;
        let speed_list = vec![4.0, 3.0, 2.0, 1.5, 1.0, 0.5, 0.25];
        let current_speed_index = 4; // default: 1.0x
        let mapper = UserEventMapper::new();
        let player = Player::new(tx.clone(), images.get(current_frame).delay_ms());
        Self {
            images,
            current_frame,
            speed_list,
            current_speed_index,
            show_details: false,
            loop_start: None,
            loop_end: None,
            clear_on_next_render: false,
            mapper,
            player,
            protocol,
            frame_step,
            max_width,
            theme,
        }
    }

    pub fn start(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<AppEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            terminal.draw(|f| self.render(f))?;

            self.update_player_state();

            match rx.recv()? {
                AppEvent::Key(key) => {
                    let user_events = self.mapper.find_events(key);
                    handle_user_events! { user_events =>
                        UserEvent::Quit => {
                            return Ok(());
                        }
                        UserEvent::SelectNextFrame => {
                            self.select_next_frame();
                        }
                        UserEvent::SelectPrevFrame => {
                            self.select_prev_frame();
                        }
                        UserEvent::SelectNextFrameStep => {
                            self.select_next_frame_step();
                        }
                        UserEvent::SelectPrevFrameStep => {
                            self.select_prev_frame_step();
                        }
                        UserEvent::SelectFirstFrame => {
                            self.select_first_frame();
                        }
                        UserEvent::SelectLastFrame => {
                            self.select_last_frame();
                        }
                        UserEvent::SelectPercentageFrame(position) => {
                            self.select_percentage_frame(*position as usize);
                        }
                        UserEvent::SelectNextSpeed => {
                            self.select_next_speed();
                        }
                        UserEvent::SelectPrevSpeed => {
                            self.select_prev_speed();
                        }
                        UserEvent::TogglePlaying => {
                            self.toggle_playing();
                        }
                        UserEvent::ToggleDetail => {
                            self.toggle_details();
                        }
                        UserEvent::SetLoopStart => {
                            self.set_loop_start();
                        }
                        UserEvent::SetLoopEnd => {
                            self.set_loop_end();
                        }
                        UserEvent::ClearLoop => {
                            self.clear_loop();
                        }
                    }
                }
                AppEvent::Resize(w, h) => {
                    self.resize(w, h);
                }
                AppEvent::NextFrame => {
                    self.select_next_frame();
                }
            }

            if self.clear_on_next_render {
                terminal.clear()?;
                self.protocol.clear();
                self.clear_on_next_render = false;
            }
        }
    }
}

impl App {
    fn select_next_frame(&mut self) {
        let (min, max) = self.loop_min_max();
        if self.current_frame == max {
            self.current_frame = min;
        } else {
            self.current_frame += 1;
        }
    }

    fn select_prev_frame(&mut self) {
        let (min, max) = self.loop_min_max();
        if self.current_frame == min {
            self.current_frame = max;
        } else {
            self.current_frame -= 1;
        }
    }

    fn select_next_frame_step(&mut self) {
        let (_, max) = self.loop_min_max();
        self.current_frame = (self.current_frame + self.frame_step).min(max);
    }

    fn select_prev_frame_step(&mut self) {
        let (min, _) = self.loop_min_max();
        self.current_frame = self.current_frame.saturating_sub(self.frame_step).max(min);
    }

    fn select_first_frame(&mut self) {
        let (min, _) = self.loop_min_max();
        self.current_frame = min;
    }

    fn select_last_frame(&mut self) {
        let (_, max) = self.loop_min_max();
        self.current_frame = max;
    }

    fn select_percentage_frame(&mut self, position: usize) {
        let (min, max) = self.loop_min_max();
        let index = (position * self.images.len()) / 10;
        self.current_frame = index.clamp(min, max);
    }

    fn select_next_speed(&mut self) {
        if self.current_speed_index == self.speed_list.len() - 1 {
            self.current_speed_index = 0;
        } else {
            self.current_speed_index += 1;
        }
    }

    fn select_prev_speed(&mut self) {
        if self.current_speed_index == 0 {
            self.current_speed_index = self.speed_list.len() - 1;
        } else {
            self.current_speed_index -= 1;
        }
    }

    fn update_player_state(&mut self) {
        let (_, adjusted_delay_ms) = self.current_delay_ms();
        self.player.set_delay_ms(adjusted_delay_ms);
    }

    fn current_delay_ms(&self) -> (u32, u32) {
        let image = self.images.get(self.current_frame);
        let delay_ms = image.delay_ms() as f32;
        let speed = self.speed_list[self.current_speed_index];
        (delay_ms as u32, (delay_ms / speed) as u32)
    }

    fn toggle_playing(&mut self) {
        if self.player.is_playing() {
            self.player.pause();
        } else {
            self.player.play();
        }
    }

    fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
        // In the case of the kitty protocol, when the drawing position changes,
        // the previous drawing remains, so we need to explicitly call the clear process.
        self.clear_on_next_render = true;
    }

    fn set_loop_start(&mut self) {
        // If the current frame is the first or last frame, it doesn't make sense to set the loop start, so clear it
        if self.current_frame == 0 || self.current_frame == self.images.len() - 1 {
            self.loop_start = None;
            return;
        }
        // If the current frame is already set as the loop start, clear it
        if let Some(frame) = self.loop_start
            && self.current_frame == frame
        {
            self.loop_start = None;
            return;
        }
        // If the current frame is set as the loop end, clear the loop end and set the loop start to the current frame
        if let Some(frame) = self.loop_end
            && self.current_frame == frame
        {
            self.loop_start = Some(self.current_frame);
            self.loop_end = None;
            return;
        }
        // Set the loop start to the current frame
        self.loop_start = Some(self.current_frame);
    }

    fn set_loop_end(&mut self) {
        // If the current frame is the first or last frame, it doesn't make sense to set the loop end, so clear it
        if self.current_frame == 0 || self.current_frame == self.images.len() - 1 {
            self.loop_end = None;
            return;
        }
        // If the current frame is already set as the loop end, clear it
        if let Some(frame) = self.loop_end
            && self.current_frame == frame
        {
            self.loop_end = None;
            return;
        }
        // If the current frame is set as the loop start, clear the loop start and set the loop end to the current frame
        if let Some(frame) = self.loop_start
            && self.current_frame == frame
        {
            self.loop_start = None;
            self.loop_end = Some(self.current_frame);
            return;
        }
        // Set the loop end to the current frame
        self.loop_end = Some(self.current_frame);
    }

    fn clear_loop(&mut self) {
        self.loop_start = None;
        self.loop_end = None;
    }

    fn loop_min_max(&self) -> (usize, usize) {
        let min = self.loop_start.unwrap_or(0);
        let max = self.loop_end.unwrap_or(self.images.len() - 1);
        (min, max)
    }

    fn resize(&mut self, _w: u16, _h: u16) {
        self.clear_on_next_render = true;
    }
}

impl App {
    fn render(&self, f: &mut Frame) {
        let area = f.area();
        let area = area.resize(Size::new(
            self.max_width.unwrap_or(area.width).min(area.width),
            area.height,
        ));

        let render_area = area.inner(Margin::new(1, 1));
        let detail_area_height = if self.show_details { 1 } else { 0 };

        let [image_area, _, gauge_area, status_area, detail_area] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(detail_area_height),
        ])
        .areas(render_area);

        self.render_image(f.buffer_mut(), image_area);
        self.render_gauge(f.buffer_mut(), gauge_area);
        self.render_status(f.buffer_mut(), status_area);
        self.render_detail(f.buffer_mut(), detail_area);
    }

    fn render_image(&self, buf: &mut Buffer, area: Rect) {
        let ScaledImageArea { l, t, r, b, w, h } =
            calc_scaled_image_area_and_dimensions(area, self.images.width(), self.images.height());

        let image = self.images.get(self.current_frame);
        let encoded = image.protocol_encoded(self.protocol, w, h);

        buf[(l, t)].set_symbol(&encoded);

        for col in l..r {
            for row in t..b {
                if col == l && row == t {
                    continue;
                }
                buf[(col, row)].set_skip(true);
            }
        }
    }

    fn render_gauge(&self, buf: &mut Buffer, area: Rect) {
        // Because a space is placed between the label and the gauge,
        // leaving the label blank will result in an extra space being placed to the left.
        // To maintain symmetry, add an extra space to the right.
        let area = area.resize(Size::new(area.width - 1, area.height));
        let ratio = (self.current_frame as f64) / ((self.images.len() - 1) as f64);
        let gauge = LineGauge::default()
            .label("")
            .filled_style(Style::default().fg(self.theme.gauge_filled_fg))
            .unfilled_style(Style::default().fg(self.theme.gauge_unfilled_fg))
            .ratio(ratio);
        gauge.render(area, buf);

        if let Some(pos) = self.loop_start {
            let loop_start_ratio = (pos as f64) / ((self.images.len() - 1) as f64);
            let loop_start_pos =
                area.left() + (area.width as f64 * loop_start_ratio).floor() as u16;
            buf[(loop_start_pos, area.top())]
                .set_symbol("[")
                .set_fg(self.theme.gauge_loop_marker_fg);
        }
        if let Some(pos) = self.loop_end {
            let loop_end_ratio = (pos as f64) / ((self.images.len() - 1) as f64);
            let loop_end_pos = area.left() + (area.width as f64 * loop_end_ratio).floor() as u16;
            buf[(loop_end_pos, area.top())]
                .set_symbol("]")
                .set_fg(self.theme.gauge_loop_marker_fg);
        }
    }

    fn render_status(&self, buf: &mut Buffer, area: Rect) {
        let area = area.inner(Margin::new(1, 0));

        let count_max_digits = self.images.max_digits();

        let show_loop_info = self.loop_start.is_some() || self.loop_end.is_some();
        let loop_info = if show_loop_info {
            let (min, max) = self.loop_min_max();
            format!("Loop: {} - {}", min + 1, max + 1,)
        } else {
            "".to_string()
        };
        let loop_info_w = loop_info.width();

        let speed_info = format!("{:.2}x", self.speed_list[self.current_speed_index]);
        let speed_info_w = speed_info.width();

        let frame_info = format!(
            "{:>count_max_digits$} / {}",
            self.current_frame + 1,
            self.images.len(),
        );
        let frame_info_w = frame_info.width();

        let filename = self.images.filename();
        let filename_w = filename.width();

        let mut space_w = (area.width as usize)
            .saturating_sub(frame_info_w)
            .saturating_sub(speed_info_w)
            .saturating_sub(3) // divider spaces
            .saturating_sub(filename_w);
        if show_loop_info {
            space_w = space_w.saturating_sub(loop_info_w).saturating_sub(3); // divider spaces
        }

        let mut spans = Vec::new();
        spans.push(filename.fg(self.theme.file_fg));
        spans.push(" ".repeat(space_w).into());
        if show_loop_info {
            spans.push(loop_info.fg(self.theme.loop_fg));
            spans.push(DIVIDER.fg(self.theme.divider_fg));
        }
        spans.push(speed_info.fg(self.theme.speed_fg));
        spans.push(DIVIDER.fg(self.theme.divider_fg));
        spans.push(frame_info.fg(self.theme.frame_fg));

        let status = Line::from(spans);
        status.render(area, buf);
    }

    fn render_detail(&self, buf: &mut Buffer, area: Rect) {
        if !self.show_details {
            return;
        }

        let area = area.inner(Margin::new(1, 0));

        let size_bytes = self.images.filesize_bytes();
        let size = format!(
            "Size: {} ({} bytes)",
            humansize::format_size(size_bytes as u64, humansize::DECIMAL),
            self.images.filesize_bytes()
        );
        let dimension = format!(
            "Dimension: {}x{}",
            self.images.width(),
            self.images.height()
        );
        let (base_delay_ms, adjusted_delay_ms) = self.current_delay_ms();
        let delay = format!("Delay: {} ms ({} ms)", base_delay_ms, adjusted_delay_ms);

        let detail = Line::from(vec![
            size.fg(self.theme.detail_fg),
            DIVIDER.fg(self.theme.divider_fg),
            dimension.fg(self.theme.detail_fg),
            DIVIDER.fg(self.theme.divider_fg),
            delay.fg(self.theme.detail_fg),
        ]);
        detail.render(area, buf);
    }
}

fn calc_scaled_image_area_and_dimensions(
    area: Rect,
    image_width: u32,
    image_height: u32,
) -> ScaledImageArea {
    // note: assume that the width-to-height ratio of the cell (font) is 1:2.
    let scale_w = (area.width as f32) / (image_width as f32);
    let scale_h = ((area.height as f32) * 2.0) / (image_height as f32);
    let scale = scale_w.min(scale_h);
    let w = (image_width as f32 * scale).floor() as u32;
    let h = ((image_height as f32 * scale) / 2.0).floor() as u32;

    let l = area.left() + ((area.width - w as u16) / 2);
    let t = area.top();
    let r = l + w as u16;
    let b = t + h as u16;
    ScaledImageArea { l, t, r, b, w, h }
}

struct ScaledImageArea {
    l: u16,
    t: u16,
    r: u16,
    b: u16,
    w: u32,
    h: u32,
}
