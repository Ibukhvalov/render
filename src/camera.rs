use glam::DVec3;
use crate::hittable::MatteSphere;
use crate::ray::{rand_in_square, Ray};

pub struct Camera {
    lower_left_corner: DVec3,

    vertical: DVec3,
    horizontal: DVec3,

    look_from: DVec3,
    look_at: DVec3,
    vup: DVec3,
}

impl Camera {
    pub fn new(look_from: DVec3, look_at: DVec3, vup: DVec3, vfov: f64, aspect: f64) -> Self {
        let w = (look_from - look_at).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        let half_height= (vfov*std::f64::consts::PI/180./2.).tan();
        let half_width = half_height*aspect;

        let lower_left_corner = look_from - w - half_height*v - half_width * u;
        let vertical = v * half_height * 2.;
        let horizontal = u * half_width * 2.;

        Self {
            lower_left_corner,
            vertical,
            horizontal,
            look_from,
            look_at,
            vup
        }
    }

    fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray {
            origin: self.look_from,
            direction: self.lower_left_corner + u*self.horizontal + v*self.vertical - self.look_from,
        }
    }

    pub fn render_to_out(&self, world: &Vec<MatteSphere>, width: u32, height: u32, samples_per_pixel: u32) {
        println!("P3\n{width} {height}\n255");


        for j in (0..height).rev() {
            for i in 0..width {
                let mut col = DVec3::ZERO;
                for _s in 0..samples_per_pixel {
                    let rnd = rand_in_square();
                    col += self.get_ray((i as f64 + rnd.x) / width as f64 , (j as f64 + rnd.y) / height as f64 )
                        .get_color(10, world);
                }
                col *= (samples_per_pixel as f64).recip();
                let ir = (col.x * 255.99) as u32;
                let ig = (col.y * 255.99) as u32;
                let ib = (col.z * 255.99) as u32;

                println!("{ir} {ig} {ib}");
            }
        }
    }
}