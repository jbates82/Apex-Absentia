//! Prepares the window icon so a missing file cannot break the build.
//!
//! `include_bytes!` resolves at compile time and fails hard if the path is
//! absent, which made a decoration into a build dependency: anybody whose
//! `assets/icon.png` had not arrived, for any reason, could not compile the
//! program at all.
//!
//! So the icon is copied into the build directory here, and a small one is
//! generated if none is present. The program then always has something to
//! include, and replacing `assets/icon.png` still works exactly as before.
use std::io::Write;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.png");

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let dest = out.join("window-icon.png");
    let src = PathBuf::from("assets/icon.png");

    if src.exists() {
        if std::fs::copy(&src, &dest).is_ok() {
            return;
        }
        println!("cargo:warning=assets/icon.png could not be read; using a generated icon");
    }

    // A ring and a keyhole, drawn rather than shipped, so this never depends
    // on a file being present.
    let png = generate();
    let mut f = std::fs::File::create(&dest).expect("write icon");
    f.write_all(&png).expect("write icon");
}

/// Writes a minimal PNG without pulling in an encoder.
fn generate() -> Vec<u8> {
    const W: u32 = 256;
    let (bg, fg) = ([0x1A, 0x1C, 0x20, 255u8], [0x7F, 0xC8, 0x9E, 255u8]);

    let mut raw = Vec::with_capacity(((W * 4 + 1) * W) as usize);
    for y in 0..W as i64 {
        raw.push(0); // no per-row filter
        for x in 0..W as i64 {
            let (cx, cy) = (x - 128, y - 128);
            let r2 = cx * cx + cy * cy;
            let mut px = bg;
            if (96 * 96..118 * 118).contains(&r2) {
                px = fg;
            }
            if cx * cx + (cy + 22) * (cy + 22) < 30 * 30 {
                px = fg;
            }
            if cx.abs() < 10 + (cy - 10) / 6 && (0..=62).contains(&(cy - 10)) {
                px = fg;
            }
            raw.extend_from_slice(&px);
        }
    }

    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.extend_from_slice(&chunk(b"IHDR", &{
        let mut h = Vec::new();
        h.extend_from_slice(&W.to_be_bytes());
        h.extend_from_slice(&W.to_be_bytes());
        h.extend_from_slice(&[8, 6, 0, 0, 0]); // 8-bit RGBA
        h
    }));
    png.extend_from_slice(&chunk(b"IDAT", &deflate_stored(&raw)));
    png.extend_from_slice(&chunk(b"IEND", &[]));
    png
}

fn chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut out = (data.len() as u32).to_be_bytes().to_vec();
    let mut body = tag.to_vec();
    body.extend_from_slice(data);
    out.extend_from_slice(&body);
    out.extend_from_slice(&crc32(&body).to_be_bytes());
    out
}

/// A zlib stream using stored (uncompressed) blocks.
///
/// Larger than a compressed one and perfectly valid, which is the right trade
/// for a fallback that exists so a build cannot fail.
fn deflate_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    for (i, block) in data.chunks(65535).enumerate() {
        let last = (i + 1) * 65535 >= data.len();
        out.push(if last { 1 } else { 0 });
        out.extend_from_slice(&(block.len() as u16).to_le_bytes());
        out.extend_from_slice(&(!(block.len() as u16)).to_le_bytes());
        out.extend_from_slice(block);
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}
