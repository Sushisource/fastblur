extern crate lodepng;

use lodepng::{Bitmap, RGBA};

fn main() {
    let image = lodepng::decode32_file("in.png");
    match image {
        Ok(img) => {
            println!("Image: {:?}", img);
            let w = img.width;
            let h = img.height;
            let blurred = blur(img);
            lodepng::encode32_file("out.png", &blurred, w, h);
        }
        Err(e) => { eprintln!("Error reading image: {}", e) }
    }
}

const BLUR_KERNEL: [[(f32, (i8, i8)); 5]; 5] = [
    [(1.0 / 256.0, (-2, 2)), (4.0 / 256.0, (-1, 2)), (6.0 / 256.0, (0, 2)), (4.0 / 256.0, (1, 2)), (1.0 / 256.0, (2, 2))],
    [(4.0 / 256.0, (-2, 1)), (16.0 / 256.0, (-1, 1)), (24.0 / 256.0, (0, 1)), (16.0 / 256.0, (1, 1)), (4.0 / 256.0, (2, 1))],
    [(6.0 / 256.0, (-2, 0)), (24.0 / 256.0, (-1, 0)), (36.0 / 256.0, (0, 0)), (24.0 / 256.0, (1, 0)), (6.0 / 256.0, (2, 0))],
    [(4.0 / 256.0, (-2, -1)), (16.0 / 256.0, (-1, -1)), (24.0 / 256.0, (0, -1)), (16.0 / 256.0, (1, -1)), (4.0 / 256.0, (2, -1))],
    [(1.0 / 256.0, (-2, -2)), (4.0 / 256.0, (-1, -2)), (6.0 / 256.0, (0, -2)), (4.0 / 256.0, (1, -2)), (1.0 / 256.0, (2, -2))],
];

fn blur(bits: Bitmap<RGBA>) -> Vec<RGBA> {
    let buff = bits.buffer;
    let mut result_buff = Vec::<RGBA>::new();

    for cur_i in 0..buff.len() {
        let pix = buff[cur_i];
        let mut accum_r = 0.0;
        let mut accum_g = 0.0;
        let mut accum_b = 0.0;

        for krow in BLUR_KERNEL.iter() {
            for (coeff, (off_x, off_y)) in krow {
                let relative_ix = lookup(*off_x as i32, *off_y as i32, cur_i as u32, bits.width as u32);
                let px_val = if relative_ix < 0 {
                    pix
                } else {
                    *buff.get(relative_ix as usize).unwrap_or(&pix)
                };
                accum_r += px_val.r as f32 * *coeff;
                accum_g += px_val.g as f32 * *coeff;
                accum_b += px_val.b as f32 * *coeff;
            }
        }
        let result = RGBA::new(accum_r as u8, accum_g as u8, accum_b as u8, 255);
        result_buff.push(result);
    }
    result_buff
}

/// Returns the index that should be used for lookup given a current pixel and an offset from it
/// in (x, y) coords. IE: Turns 2d relative space into linear absolute space.
#[inline]
fn lookup(offset_x: i32, offset_y: i32, curpx_index: u32, img_width: u32) -> i32 {
    let (cur_x, cur_y) = ((curpx_index % img_width) as i32,
                          (curpx_index / img_width) as i32);
//    println!("curx {} cury {} offx {} offy {}", cur_x, cur_y, offset_x, offset_y);
//    println!("res {}",
//             (cur_x + offset_x) + (cur_y + offset_y) * (img_width as i32));

    (cur_x + offset_x) + (cur_y + offset_y) * (img_width as i32)
}