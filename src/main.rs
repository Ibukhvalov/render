use glam::Vec3;
use crate::camera::Camera;
use hittable::matte_sphere::MatteSphere;
use crate::hittable::{ HittableSurfaces};

mod ray;
mod hittable;
mod camera;
mod interval;
mod util;

use crate::hittable::fog::Fog;

fn main() {
    const ASPECT: f32 = 16./9.;
    const IMG_WIDTH: u32 = 200;
    const IMG_HEIGHT: u32 = (IMG_WIDTH as f32 / ASPECT) as u32;


    /*
    let filename = std::env::args()
        .nth(1)
        .expect("Missing VDB filename as first argument");

    let f = File::open(filename).unwrap();
    let mut vdb_reader = VdbReader::new(BufReader::new(f)).unwrap();
    let grid_names = vdb_reader.available_grids();

    let grid_to_load = grid_names.first().cloned().unwrap_or(String::new());


    let grid = vdb_reader.read_grid::<half::f16>(&grid_to_load).unwrap();

    let world: Vec<HittableSurfaces> = grid
        .iter()
        .map(|(pos, voxel, level)| {
            let pos_vec3 = Vec3::new(pos.x, pos.y, pos.z);
            HittableSurfaces::MatteSphere(MatteSphere::new(
                (pos_vec3 + level.scale()) * 0.1,
                level.scale()*0.1,
                Vec3::splat(0.3),
            ))
        })
        .collect();
    */
    let red_sun = HittableSurfaces::MatteSphere(MatteSphere::new(Vec3::Y, 1., Vec3::new(0.5,0.8,0.8)));
    let sun = HittableSurfaces::MatteSphere(MatteSphere::new(Vec3::new(0.,2.,-200.), 100., Vec3::new(1.,0.9,0.4)));
    let cloud1 = HittableSurfaces::MatteSphere(MatteSphere::new(Vec3::new(3.,3.,1.), 1., Vec3::splat(0.9)));

    let world = vec![
        HittableSurfaces::MatteSphere(MatteSphere::new(Vec3::new(0.,-1000.,0.), 1000., Vec3::new(0.2,0.5,0.9))),
        sun,
        cloud1,
        HittableSurfaces::Fog(Fog::new(red_sun, 0.35)),
    ];



    let camera = Camera::new(Vec3::new(0.,1., 3.), Vec3::new(0.,1.5,0.), Vec3::Y, 90., ASPECT);

    camera.render_to_out(world, IMG_WIDTH, IMG_HEIGHT, 20);



}
