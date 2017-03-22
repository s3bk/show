extern crate piston_window;
extern crate image;
extern crate input;

// fractal dependencies
extern crate num_complex;
extern crate palette;
extern crate simd;

use image::{Rgba, RgbaImage};
use num_complex::Complex;

trait Visible {
    fn update(&mut self, t: f64) -> &RgbaImage;
    fn cursor(&mut self, x: f64, y: f64) {}
}

fn show<V: Visible>(mut v: V) {
    use piston_window::*;
    
    let (mut window, mut texture) = {
        let img0 = v.update(0.0);
        
        let opengl = OpenGL::V3_2;
        let mut window: PistonWindow =
            WindowSettings::new("piston: image", [img0.width(), img0.height()])
            .exit_on_esc(true)
            .opengl(opengl)
            .build()
            .unwrap();
        
        let texture = Texture::from_image(
            &mut window.factory,
            img0,
            &TextureSettings::new()
        ).unwrap();
        (window, texture)
    };
    
    let mut t = 0.0;
    //window.set_lazy(true);
    while let Some(e) = window.next() {
        use input::{Input, Motion};
        println!("{:3.5} {:?}", t, e);
        match e {
            Input::Render(args) => {
                t += args.ext_dt;
                let img = v.update(t);
                texture.update(&mut window.encoder, img).unwrap();
                window.draw_2d(&e, |c, g| {
                    clear([1.0; 4], g);
                    image(&texture, c.transform, g);
                });
            },
            Input::Move(Motion::MouseCursor(x, y)) => v.cursor(x, y),
            Input::Update(args) => t += args.dt,
            _ => ()
        }
    }
}

struct FractalSettings {
    width: u32,
    height: u32,
    n_iter: u32,
}

use std::time::{Instant, Duration};
use palette::{Gradient, Lch};
struct Fractal {
    img: RgbaImage,
    s: FractalSettings,
    colormap: Vec<Rgba<u8>>,
    center: Complex<f32>,
    c: Complex<f32>
}
impl Fractal {
    fn new(s: FractalSettings) -> Fractal {
        use std::f32::consts::PI;
        use palette::{Lch, LabHue, IntoColor};
        
        let k = s.n_iter as f32;
        let gradient = Gradient::with_domain(vec![
            (0.0 * k, Lch::new(0.0, 1., LabHue::from_radians(-2. * PI / 3.))),
            (0.3 * k, Lch::new(0.2, 1., LabHue::from_radians(-1. * PI / 3.))),
            (0.6 * k, Lch::new(0.5, 1., LabHue::from_radians(0.))),
            (0.8 * k, Lch::new(0.8, 1., LabHue::from_radians(PI / 3.))),
            (1.0 * k, Lch::new(1.0, 0., LabHue::from_radians(PI / 3.)))
        ]);
        
        let colormap = (0 .. s.n_iter).map(|i| {
            let (r, g, b) = gradient.get(i as f32).into_rgb().to_pixel();
            image::Rgba([r, g, b, 255])
        }).collect();
        
        // Create a new ImgBuf with width: imgx and height: imgy
        let imgbuf = RgbaImage::new(s.width, s.height);
    
        Fractal {
            img: imgbuf,
            s: s,
            colormap: colormap,
            center: Complex::new(-0.0, 0.0),
            c: Complex::new(-0.6, 0.4)
        }
    }
}

macro_rules! each {
    ($simd:expr; $i:ident ( $($counter:expr),* ), $body:block) => (
        $simd (
            $( { let $i = $counter; $body } ),*
        )
    )
}

impl Visible for Fractal {
    fn cursor(&mut self, x: f64, y: f64) {
        self.c = Complex::new(
            x as f32 / self.s.width as f32, 
            y as f32 / self.s.height as f32
        );
    }
    fn update(&mut self, t: f64) -> &RgbaImage {
        use simd::x86::avx::{f32x8, u32x8, bool32ix8};
        let zoom = (0.9f32).powf(t as f32);
        
        let scale = 2.0 * zoom;
        let off = self.center - Complex::new(scale, scale);
        let scalex = 2.0 * scale / self.s.width as f32;
        let scaley = 2.0 * scale / self.s.height as f32;
        
        let limit = f32x8::splat(4.0);
        for y in 0 .. self.s.height {
            for x in 0 .. self.s.width / 8 {
                let mut z_im = f32x8::splat(y as f32 * scaley + off.im);
                let mut z_re = each!(f32x8::new ; i (0, 1, 2, 3, 4, 5, 6, 7), {
                    (x*8 + i) as f32 * scalex + off.re
                });

                let mut i = u32x8::splat(0);
                let mut mask = bool32ix8::splat(false);
                
                for t in 0 .. self.s.n_iter {
                    // square z
                    let z_re2 = z_re * z_re;
                    let z_im2 = z_im * z_im;
                    
                    mask = mask | (z_re2 + z_im2).gt(limit).to_i();
                    if mask.all() {
                        break;
                    }
                    i = mask.select(i, u32x8::splat(t));
                    
                    z_im = z_re * z_im * f32x8::splat(2.0);
                    z_re = z_re2 - z_im2;
                    
                    z_re = z_re + f32x8::splat(self.c.re);
                    z_im = z_im + f32x8::splat(self.c.im);
                }

                // Create an 8bit pixel of type Luma and value i
                // and assign in to the pixel at position (x, y)
                
                for j in 0 .. 8 {
                    *self.img.get_pixel_mut(x*8 + j, y) = self.colormap[i.extract(j) as usize];
                }
            }
        }
    
        &self.img
    }
}

fn main() {
    let f = Fractal::new(FractalSettings {
        width: 600,
        height: 400,
        n_iter: 200
    });

    show(f);
}
