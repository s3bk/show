extern crate piston_window;
extern crate image;
extern crate input;
extern crate graphics;

use image::RgbaImage;

pub enum Rotation {
    R0,
    R90,
    R180,
    R270
}

pub trait Visible : Sized {
    fn update(&mut self, t: f64) -> &RgbaImage;
    fn maybe_update(&mut self, t: f64) -> Option<&RgbaImage> {
        Some(self.update(t))
    }
    fn cursor(&mut self, x: f64, y: f64) {}

    fn show(&mut self, rot: Rotation) {
        use piston_window::*;

        let (mut window, mut texture) = {
            let img0 = self.update(0.0);

            let (width, height) = match rot {
                Rotation::R0 | Rotation::R180 => (img0.width(), img0.height()),
                Rotation::R90 | Rotation::R270 => (img0.height(), img0.width())
            };

            let opengl = OpenGL::V3_2;
            let mut window: PistonWindow = WindowSettings::new("piston: image", [width, height])
                .exit_on_esc(true)
                .opengl(opengl)
                .decorated(true)
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
        while let Some(e) = window.next() {
            use input::{Input, Motion};

            println!("{:3.5} {:?}", t, e);
            match e {
                Input::Render(args) => {
                    t += args.ext_dt;
                    let img = self.update(t);
                    texture.update(&mut window.encoder, img).unwrap();
                    window.draw_2d(&e, |c, g| {
                        clear([1.0; 4], g);
                        let (w, h) = (img.width() as f64, img.height() as f64);
                        let transform = match rot {
                            Rotation::R0 =>   [[ 2./w,    0., -1.], [    0., -2./h,   1.]],
                            Rotation::R90 =>  [[ 0.,    2./h, -1.], [ 2./w,     0.,  -1.]],
                            Rotation::R180 => [[-2./w,    0.,  1.], [    0.,  2./h,  -1.]],
                            Rotation::R270 => [[ 0.,   -2./h,  1.], [-2./w,     0.,   1.]],
                        };
                        image(&texture, transform, g);
                    });
                },
                Input::Move(Motion::MouseCursor(x, y)) => self.cursor(x, y),
                Input::Update(args) => t += args.dt,
                _ => ()
            }
        }
    }
}

impl Visible for RgbaImage {
    fn update(&mut self, t: f64) -> &RgbaImage {
        self
    }
}
