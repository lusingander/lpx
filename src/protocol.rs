use std::env;

pub fn auto_detect() -> ImageProtocol {
    // https://sw.kovidgoyal.net/kitty/glossary/#envvar-KITTY_WINDOW_ID
    if env::var("KITTY_WINDOW_ID").is_ok() {
        return ImageProtocol::Kitty;
    }
    // https://ghostty.org/docs/help/terminfo
    if env::var("TERM").is_ok_and(|t| t == "xterm-ghostty") {
        return ImageProtocol::Kitty;
    }
    ImageProtocol::Iterm2
}

#[derive(Debug, Clone, Copy)]
pub enum ImageProtocol {
    Iterm2,
    Kitty,
}

impl ImageProtocol {
    pub fn encode(&self, base64_encoded: &str, w: u32, h: u32, bytes_len: usize) -> String {
        match self {
            ImageProtocol::Iterm2 => iterm2_encode(base64_encoded, w, h, bytes_len),
            ImageProtocol::Kitty => kitty_encode(base64_encoded, w, h),
        }
    }

    pub fn clear(&self) {
        match self {
            ImageProtocol::Iterm2 => {}
            ImageProtocol::Kitty => print!("\x1b_Ga=d,d=A;\x1b\\"),
        }
    }
}

fn iterm2_encode(base64_encoded: &str, _w: u32, h: u32, bytes_len: usize) -> String {
    format!(
        "\x1b]1337;File=size={};height={};preserveAspectRatio=1;inline=1:{}\u{0007}",
        bytes_len, h, base64_encoded,
    )
}

fn kitty_encode(base64_encoded: &str, _w: u32, h: u32) -> String {
    let chunk_size = 4096;

    let mut s = String::new();

    let chunks = base64_encoded.as_bytes().chunks(chunk_size);
    let total_chunks = chunks.len();

    s.push_str("\x1b_Ga=d,d=C;\x1b\\");
    for (i, chunk) in chunks.enumerate() {
        s.push_str("\x1b_G");
        if i == 0 {
            s.push_str(&format!("a=T,f=100,r={h},"));
        }
        if i < total_chunks - 1 {
            s.push_str("m=1;");
        } else {
            s.push_str("m=0;");
        }
        s.push_str(std::str::from_utf8(chunk).unwrap());
        s.push_str("\x1b\\");
    }

    s
}
