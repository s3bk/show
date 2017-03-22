extern crate piston_window;
extern crate image;
extern crate input;

use image::RgbaImage;


pub trait Visible : Sized {
    fn update(&mut self, t: f64) -> &RgbaImage;
    fn cursor(&mut self, x: f64, y: f64) {}

    fn show(mut self) {
        use piston_window::*;
        
        let (mut window, mut texture) = {
            let img0 = self.update(0.0);
            
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
                    let img = self.update(t);
                    texture.update(&mut window.encoder, img).unwrap();
                    window.draw_2d(&e, |c, g| {
                        clear([1.0; 4], g);
                        image(&texture, c.transform, g);
                    });
                },
                Input::Move(Motion::MouseCursor(x, y)) => self.cursor(x, y),
                Input::Update(args) => t += args.dt,
                _ => ()
            }
        }
    }
}
