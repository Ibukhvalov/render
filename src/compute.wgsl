@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> color: vec4f;

@group(0) @binding(2)
var<uniform> camera_to_world: mat4x4f;

const width = 800.0;
const height = 600.0;

const ratio = width/height;

struct Ray {
    origin: vec4f,
    direction: vec4f,
}

struct Sphere {
    origin: vec3f,
    radius: f32,
}



fn hit_sphere(sphere: Sphere, ray: Ray) -> bool {
    let oc = ray.origin.xyz - sphere.origin;
    let a = dot(ray.direction.xyz, ray.direction.xyz);
    let b = 2.0 * dot(oc, ray.direction.xyz);
    let c = dot(oc, oc) - sphere.radius*sphere.radius;

    return (b*b - 4*a*c) > 0;
}


fn get_ray(u: f32, v: f32) -> Ray {
    return Ray(camera_to_world * vec4f(0.0, 0.0, 0.0, 1.0),
        camera_to_world * vec4f((u*2.0-1.0) * ratio, -(v*2.0-1.0), 1.0, 0.0));
}


@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    let sphere = Sphere(vec3f(0.0), 20.0);
    let u = f32(global_id.x) / width; 
    let v = f32(global_id.y) / height; 
    let ray = get_ray(u,v);
    
    
    if(hit_sphere(sphere, ray)) {
        textureStore(output_texture, global_id.xy, vec4f(1.0,0.0,0.0,1.0));
    } else {
        let t = (ray.direction.y + 1.0) * 0.5;
        let background = (1.0 - t) * vec3(1.0) + t * vec3(0.5,0.7,1.0);
        textureStore(output_texture, global_id.xy, vec4f(background, 1.0));
    }


}
