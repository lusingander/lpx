use std::{fs::File, io::Cursor};

use base64::Engine;
use rayon::prelude::*;

use crate::protocol::ImageProtocol;

pub struct Images {
    images: Vec<Image>,
    filename: String,
    size_bytes: usize,
    width: u32,
    height: u32,
}

pub struct Image {
    bytes: Vec<u8>,
    base64_encoded: String,
    delay: u16,
}

impl Images {
    pub fn load(file: File, filename: &str) -> Result<Images, Box<dyn std::error::Error>> {
        let size_bytes = file.metadata()?.len() as usize;

        let mut decode_options = gif::DecodeOptions::new();
        decode_options.set_color_output(gif::ColorOutput::Indexed);

        let mut decoder = decode_options.read_info(file)?;
        let mut screen = gif_dispose::Screen::new_decoder(&decoder);

        let width = screen.width() as u32;
        let height = screen.height() as u32;

        let mut frames = Vec::new();
        while let Some(frame) = decoder.read_next_frame()? {
            screen.blit_frame(frame)?;

            let rgba = screen.pixels_rgba();
            let bytes = bytemuck::cast_slice(rgba.buf());

            let frame = Frame {
                rgba_bytes: bytes.to_vec(),
                delay: frame.delay,
            };
            frames.push(frame);
        }

        let images = frames
            .into_par_iter()
            .map(|f| frame_to_png_bytes(&f, width, height).map(|b| (f, b)))
            .map(|r| r.map(|(f, b)| (to_base64_str(&b), f, b)))
            .map(|r| {
                r.map(|(base64_encoded, frame, bytes)| Image {
                    bytes,
                    base64_encoded,
                    delay: frame.delay,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Images {
            images,
            filename: filename.into(),
            size_bytes,
            width,
            height,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn len(&self) -> usize {
        self.images.len()
    }

    pub fn filesize_bytes(&self) -> usize {
        self.size_bytes
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn max_digits(&self) -> usize {
        let mut n = self.len();
        let mut c = 0;
        while n > 0 {
            n /= 10;
            c += 1;
        }
        c
    }

    pub fn get(&self, index: usize) -> &Image {
        &self.images[index]
    }
}

impl Image {
    pub fn protocol_encoded(&self, protocol: ImageProtocol, width: u32, height: u32) -> String {
        protocol.encode(&self.base64_encoded, width, height, self.bytes.len())
    }

    pub fn delay_ms(&self) -> u32 {
        (self.delay as u32) * 10
    }
}

struct Frame {
    rgba_bytes: Vec<u8>,
    delay: u16,
}

fn frame_to_png_bytes(frame: &Frame, width: u32, height: u32) -> Result<Vec<u8>, String> {
    let mut buf = Cursor::new(Vec::new());
    image::write_buffer_with_format(
        &mut buf,
        &frame.rgba_bytes,
        width,
        height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}

fn to_base64_str(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
