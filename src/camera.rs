use crate::hittable::HittableSurfaces;
use crate::ray::Ray;
use crate::util::rand_in_square;
use glam::Vec3;
use indicatif::ProgressBar;

pub struct Camera {
    lower_left_corner: Vec3,

    vertical: Vec3,
    horizontal: Vec3,

    look_from: Vec3,
    look_at: Vec3,
    vup: Vec3,
}

impl Camera {
    pub fn new(look_from: Vec3, look_at: Vec3, vup: Vec3, vfov: f32, aspect: f32) -> Self {
        let w = (look_from - look_at).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        let half_height = (vfov * std::f32::consts::PI / 180. / 2.).tan();
        let half_width = half_height * aspect;

        let lower_left_corner = look_from - w - half_height * v - half_width * u;
        let vertical = v * half_height * 2.;
        let horizontal = u * half_width * 2.;

        Self {
            lower_left_corner,
            vertical,
            horizontal,
            look_from,
            look_at,
            vup,
        }
    }

    fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray {
            origin: self.look_from,
            direction: self.lower_left_corner + u * self.horizontal + v * self.vertical
                - self.look_from,
        }
    }

    pub fn render_to_out(
        &self,
        world: Vec<HittableSurfaces>,
        width: u32,
        height: u32,
        samples_per_pixel: u32,
    ) {
        println!("P3\n{width} {height}\n255");

        eprintln!("[2/2]🔺 Rendering...");
        let number_of_pixels = height * width;
        let pb = ProgressBar::new(number_of_pixels as u64);

        for j in (0..height).rev() {
            for i in 0..width {
                let mut col = Vec3::ZERO;
                for _s in 0..samples_per_pixel {
                    let rnd = rand_in_square();
                    col += self
                        .get_ray(
                            (i as f32 + rnd.x) / width as f32,
                            (j as f32 + rnd.y) / height as f32,
                        )
                        .get_color(7, &world);
                }
                col *= (samples_per_pixel as f32).recip();
                let ir = (col.x * 255.99) as u32;
                let ig = (col.y * 255.99) as u32;
                let ib = (col.z * 255.99) as u32;

                println!("{ir} {ig} {ib}");
            }
            pb.inc(width as u64);
        }
        pb.finish_with_message("✅ Complete!");
    }
}
