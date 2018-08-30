extern crate image;
#[macro_use]
extern crate itertools;
extern crate rayon;

use image::RgbaImage;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Must pass exactly two args - input filename and output filename.");
        std::process::exit(-1);
    }
    let infile = &args[1];
    let outfile = &args[2];
    let image = image::open(infile);
    match image {
        Ok(img) => {
            let converted = img.to_rgba();
            println!("Convert done.");
            let blur_s = Instant::now();
            let blurred = blur(converted);
            println!("Blur done in {:?}.", blur_s.elapsed());
            blurred.save(outfile)?;
            println!("{} written.", outfile);
        }
        Err(e) => { eprintln!("Error reading image: {}", e) }
    }
    Ok(())
}

fn blur(bits: RgbaImage) -> RgbaImage {
    let imgw = bits.width();
    let imgh = bits.height();

    let pixels = bits.pixels();
    let mut rchan: Vec<u8> = Vec::new();
    let mut gchan: Vec<u8> = Vec::new();
    let mut bchan: Vec<u8> = Vec::new();
    for pix in pixels {
        rchan.push(pix.data[0]);
        gchan.push(pix.data[1]);
        bchan.push(pix.data[2]);
    }
    let sigma = 5;
    let mut resr: Vec<u8> = Vec::new();
    let mut resg: Vec<u8> = Vec::new();
    let mut resb: Vec<u8> = Vec::new();
    rayon::scope(|s| {
        s.spawn(|_| resr = new_blur(rchan, imgw, imgh, sigma));
        s.spawn(|_| resg = new_blur(gchan, imgw, imgh, sigma));
        s.spawn(|_| resb = new_blur(bchan, imgw, imgh, sigma));
    });
    let mut res: Vec<u8> = Vec::new();
    for (r, g, b) in izip!(resr.as_slice(), resg.as_slice(), resb.as_slice()) {
        res.push(*r);
        res.push(*g);
        res.push(*b);
        res.push(255); //alpha
    }
    return RgbaImage::from_raw(imgw, imgh, res).unwrap();
}

fn boxes_for_gauss(sigma: f32, n: f32) -> Vec<f32> {
    let w_ideal = ((12.0 * sigma * sigma / n) + 1.0).sqrt();  // Ideal averaging filter width
    let mut wl = w_ideal.floor();
    if wl % 2.0 == 0.0 {
        wl -= 1.0;
    }
    let wu = wl + 2.0;

    let m_ideal = (12.0 * sigma * sigma - n * wl * wl - 4.0 * n * wl - 3.0 * n) / (-4.0 * wl - 4.0);
    let m = m_ideal.round();

    return (0..n as u16).into_iter().map(|i| if (i as f32) < m { wl } else { wu }).collect();
}

fn new_blur(source: Vec<u8>, w: u32, h: u32, r: usize) -> Vec<u8> {
    let bxs = boxes_for_gauss(r as f32, 3.0);
    let res = box_blur(source, w, h, (bxs[0] as usize - 1) / 2);
    let res2 = box_blur(res, w, h, (bxs[1] as usize - 1) / 2);
    box_blur(res2, w, h, (bxs[2] as usize - 1) / 2)
}

fn box_blur(source: Vec<u8>, w: u32, h: u32, r: usize) -> Vec<u8> {
    let hblur = box_blur_h(source, w, h, r);
    box_blur_v(hblur, w, h, r)
}

fn box_blur_h(source: Vec<u8>, w: u32, h: u32, r: usize) -> Vec<u8> {
    let iarr = 1.0 / (r + r + 1) as f32;
    let w = w as usize;
    let mut output: Vec<u8> = vec![0; source.len()];

    for i in 0..h {
        let mut ti: usize = w * i as usize;
        let mut li: usize = ti;
        let mut ri: usize = ti + r as usize;
        let fv = source[ti] as f32;
        let lv = source[ti + w as usize - 1] as f32;
        let mut val = (r + 1) as f32 * fv;
        for j in 0..r { val += source[ti + j] as f32; }
        for _ in 0..r + 1 {
            val += source[ri] as f32 - fv;
            output[ti] = (val * iarr) as u8;
            ri += 1;
            ti += 1;
        }
        for _ in r + 1..w - r {
            val += source[ri] as f32 - source[li] as f32;
            output[ti] = (val * iarr) as u8;
            ri += 1;
            li += 1;
            ti += 1;
        }
        for _ in w - r..w {
            val += lv - source[li] as f32;
            output[ti] = (val * iarr) as u8;
            li += 1;
            ti += 1;
        }
    };

    output
}

fn box_blur_v(source: Vec<u8>, w: u32, h: u32, r: usize) -> Vec<u8> {
    let iarr = 1.0 / (r + r + 1) as f32;
    let w = w as usize;
    let h = h as usize;
    let mut output: Vec<u8> = vec![0; source.len()];

    for i in 0..w {
        let mut ti: usize = i as usize;
        let mut li: usize = ti;
        let mut ri: usize = ti + r * w;
        let fv = source[ti] as f32;
        let lv = source[ti + w * (h - 1)] as f32;
        let mut val = (r + 1) as f32 * fv;
        for j in 0..r { val += source[ti + j * w] as f32; }
        for _ in 0..r + 1 {
            val += source[ri] as f32 - fv;
            output[ti] = (val * iarr) as u8;
            ri += w;
            ti += w;
        }
        for _ in r + 1..h - r {
            val += source[ri] as f32 - source[li] as f32;
            output[ti] = (val * iarr) as u8;
            ri += w;
            li += w;
            ti += w;
        }
        for _ in h - r..h {
            val += lv - source[li] as f32;
            output[ti] = (val * iarr) as u8;
            li += w;
            ti += w;
        }
    };

    output
}
