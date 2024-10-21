use glam::DVec3;
use crate::camera::Camera;
use crate::hittable::MatteSphere;

mod ray;
mod hittable;
mod camera;

fn main() {
    const ASPECT: f64 = 16./9.;
    const IMG_WIDTH: u32 = 500;
    const IMG_HEIGHT: u32 = (IMG_WIDTH as f64 / ASPECT) as u32;


    let mut world = vec![];
    world.push(MatteSphere::new(DVec3::new(0.,-100.,0.), 100., DVec3::splat(0.6)));
    world.push(MatteSphere::new(DVec3::new(0.3,1.,0.), 1., DVec3::new(0.9,0.4,0.4)));
    world.push(MatteSphere::new(DVec3::new(-0.3,1.,0.), 1., DVec3::new(0.9,0.4,0.4)));

    let camera = Camera::new(DVec3::new(0.,0.5,-3.), DVec3::ZERO, DVec3::Y, 80., 16./9.);

    camera.render_to_out(&world, IMG_WIDTH, IMG_HEIGHT, 50);
}
