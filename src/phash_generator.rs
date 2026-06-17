use std::fs::File;
use std::io::Read;
use std::path::Path;

const WIDTH: usize = 32;
const HEIGHT: usize = 32;
const HASH_DCT_SIZE: usize = 8;

fn read_raw_frame(path: &Path) -> [u8; WIDTH * HEIGHT] {
    let mut buf = [0u8; WIDTH * HEIGHT];
    let mut file = File::open(path).expect("Cannot open .raw image");
    file.read_exact(&mut buf)
        .expect("Cannot read bytes of the image");

    buf
}

// Computes a 64-bit pHash compatible with the original C++ version
fn compute_phash(frame: &[u8; WIDTH * HEIGHT]) -> u64 {
    let mut dct = [[0f32; HASH_DCT_SIZE]; HASH_DCT_SIZE];

    // Precompute cosine tables
    let mut cos_x = [[0f32; HASH_DCT_SIZE]; WIDTH];
    let mut cos_y = [[0f32; HASH_DCT_SIZE]; HEIGHT];

    for x in 0..WIDTH {
        for u in 0..HASH_DCT_SIZE {
            cos_x[x][u] = ((2 * x + 1) as f32 * u as f32 * std::f32::consts::PI / 64.0).cos();
        }
    }

    for y in 0..HEIGHT {
        for v in 0..HASH_DCT_SIZE {
            cos_y[y][v] = ((2 * y + 1) as f32 * v as f32 * std::f32::consts::PI / 64.0).cos();
        }
    }

    // Compute top-left HASHSIZExHASHSIZE DCT coefficients
    for u in 0..HASH_DCT_SIZE {
        for v in 0..HASH_DCT_SIZE {
            let mut sum = 0f32;
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    sum += frame[y * WIDTH + x] as f32
                        * cos_x[x][u]
                        * cos_y[y][v];
                }
            }
            let cu = if u == 0 {
                std::f32::consts::FRAC_1_SQRT_2
            } else {
                1.0
            };
            
            let cv = if v == 0 {
                std::f32::consts::FRAC_1_SQRT_2
            } else {
                1.0
            };
            
            dct[u][v] = 0.25 * cu * cv * sum;
        }
    }

    // Compute average of HASHSIZExHASHSIZE DCT block
    let mut total = 0f32;
    for u in 0..HASH_DCT_SIZE {
        for v in 0..HASH_DCT_SIZE {
            total += dct[u][v];
        }
    }
    let avg = total / (HASH_DCT_SIZE * HASH_DCT_SIZE) as f32;

    // Build 64-bit hash
    let mut hash = 0u64;
    for u in 0..HASH_DCT_SIZE {
        for v in 0..HASH_DCT_SIZE {
            hash <<= 1;
            if dct[u][v] > avg {
                hash |= 1;
            }
        }
    }
    hash
}

pub fn generate_phash(file_path: &str) -> u64 {
    let path = Path::new(file_path);
    let frame = read_raw_frame(path);
    compute_phash(&frame)
}

// Calculates the difference between two given hashes
pub fn hamming(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}
