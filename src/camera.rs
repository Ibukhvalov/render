use crate::hittable::grid::VolumeGrid;
use crate::interval::Interval;
use crate::ray::Ray;
use crate::util::rand_in_square;
use glam::Vec3;
use indicatif::ProgressBar;
use rayon::prelude::*;

pub struct Camera {
    top_left_corner: Vec3,

    vertical: Vec3,
    horizontal: Vec3,

    look_from: Vec3,
}

impl Camera {
    pub fn new(look_from: Vec3, look_at: Vec3, vup: Vec3, vfov: f32, aspect: f32) -> Self {
        let w = (look_from - look_at).normalize();
        let u = vup.cross(w).normalize();
        let v = u.cross(w);

        let half_height = (vfov * std::f32::consts::PI / 180. / 2.).tan();
        let half_width = half_height * aspect;

        let top_left_corner = look_from - w - half_height * v - half_width * u;
        let vertical = v * half_height * 2.;
        let horizontal = u * half_width * 2.;

        Self {
            top_left_corner,
            vertical,
            horizontal,
            look_from,
        }
    }

    // #->(u)#
    // I######
    // v######
    // (v)####
    fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.look_from,
            self.top_left_corner + u * self.horizontal + v * self.vertical - self.look_from,
        )
    }

    pub fn render_to_out(
        &self,
        world: &VolumeGrid,
        width: u32,
        height: u32,
        samples_per_pixel: u32,
    ) {
        let mut imgbuf = image::ImageBuffer::new(width, height);

        eprintln!("[2/3] 🔺 Rendering...");
        let number_of_pixels = height * width;
        let pb = ProgressBar::new(number_of_pixels as u64);

        let bgcolor = Vec3::new(0.572f32, 0.772f32, 0.921f32);
        let border = Interval::new(0., 0.9999);
        for (i, j, pixel) in imgbuf.enumerate_pixels_mut() {
            let col = (0..samples_per_pixel)
                .into_par_iter()
                .map(|_s| {
                    let rnd = rand_in_square();
                    self.get_ray(
                        (i as f32 + rnd.x) / width as f32,
                        (j as f32 + rnd.y) / height as f32,
                    )
                    .get_color(world, &bgcolor)
                })
                .reduce(|| Vec3::ZERO, |accum, color| accum + color)
                * (samples_per_pixel as f32).recip();

            let col_u8: [u8; 3] = [
                (border.clamp(col.x) * 255.99) as u8,
                (border.clamp(col.y) * 255.99) as u8,
                (border.clamp(col.z) * 255.99) as u8,
            ];
            *pixel = image::Rgb(col_u8);
            pb.inc(1);
        }
        pb.finish();

        let path = "data/out.png";
        eprintln!("[3/3] 🖼️ Saving an image -> {path}");
        imgbuf.save(path).unwrap();
    }
}
