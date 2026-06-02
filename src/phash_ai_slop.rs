//THIS CODE WAS GENERATED WITH CHAT GPT
use std::fs::File;
use std::io::Read;
use std::path::Path;

const WIDTH: usize = 32;
const HEIGHT: usize = 32;

/// Reads a 32x32 raw grayscale frame into a [u8; 1024] array
fn read_raw_frame(path: &Path) -> [u8; WIDTH * HEIGHT] {
    let mut buf = [0u8; WIDTH * HEIGHT];
    let mut file = File::open(path).expect("Cannot open raw file");
    file.read_exact(&mut buf).expect("Cannot read 32x32 bytes");
    buf
}

/// Computes a 64-bit pHash compatible with your friend's C++ version
fn compute_phash(frame: &[u8; WIDTH * HEIGHT]) -> u64 {
    let mut dct = [[0f32; 8]; 8];

    // Precompute cosine tables
    let mut cos_x = [[0f32; 8]; WIDTH];
    let mut cos_y = [[0f32; 8]; HEIGHT];

    for x in 0..WIDTH {
        for u in 0..8 {
            cos_x[x][u] = ((2 * x + 1) as f32 * u as f32 * std::f32::consts::PI / 64.0).cos();
        }
    }

    for y in 0..HEIGHT {
        for v in 0..8 {
            cos_y[y][v] = ((2 * y + 1) as f32 * v as f32 * std::f32::consts::PI / 64.0).cos();
        }
    }

    // Compute top-left 8x8 DCT coefficients
    for u in 0..8 {
        for v in 0..8 {
            let mut sum = 0f32;
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    let pixel = frame[y * WIDTH + x] as f32;
                    sum += pixel * cos_x[x][u] * cos_y[y][v];
                }
            }
            let cu = if u == 0 { 1.0 / 2f32.sqrt() } else { 1.0 };
            let cv = if v == 0 { 1.0 / 2f32.sqrt() } else { 1.0 };
            dct[u][v] = 0.25 * cu * cv * sum;
        }
    }

    // Compute average of 8x8 DCT block
    let mut total = 0f32;
    for u in 0..8 {
        for v in 0..8 {
            total += dct[u][v];
        }
    }
    let avg = total / 64.0;

    // Build 64-bit hash
    let mut hash = 0u64;
    for u in 0..8 {
        for v in 0..8 {
            hash <<= 1;
            if dct[u][v] > avg {
                hash |= 1;
            }
        }
    }
    hash
}

pub fn generate_phash(file_path: &str) -> u64 {
    let path = Path::new(file_path); // your raw frame path
    let frame = read_raw_frame(path);
    compute_phash(&frame)
}
