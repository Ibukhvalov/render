@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<storage, read> volume_grid: VolumeGridStatic;

@group(0) @binding(2)
var<storage, read> weights: array<u32>;

@group(0) @binding(3)
var<uniform> uniforms: Uniforms;

const width = 800.0;
const height = 600.0;

const INF = 99999.0;

const ratio = width / height;

const PI: f32 = 3.14159265358979323846;

const BASE_WEIGHT: f32 = 0.1;


struct Uniforms {
    color: vec4f,
    camera_to_world: mat4x4f,
    light_dir: vec4f,
    light_col: vec4f,
    absorption: f32,
    scattering: f32,
    g: f32,
    step_size: f32,
}

struct Ray {
    origin: vec4f,
    direction: vec4f,
}

struct Aabb {
    min: vec4f,
    max: vec4f,
}

struct VolumeGridStatic {
    size: vec4u,
    shift: vec4i,
    bbox: Aabb,
}

struct Sphere {
    origin: vec3f,
    radius: f32,
}

struct Interval {
    start: f32,
    end: f32,
}

struct RayRecord {
    transparency: f32,
    color: vec3f,
}


fn ray_at(ray: Ray, t: f32) -> vec3f {
    return ray.origin.xyz + ray.direction.xyz * t;
}

fn create_ray(origin: vec3f, dir: vec3f) -> Ray {
    return Ray(vec4f(origin, 1.0), vec4f(normalize(dir), 0.0));
}



fn hit_aabb(aabb: Aabb, ray: Ray) -> Interval {
    let ray_orig = ray.origin.xyz;
    let ray_dir = ray.direction.xyz;

    var interval = Interval(0.0, INF);

    for (var axis: u32 = 0; axis < 3; axis++) {

        let inv_d = 1.0 / ray_dir[axis];
        var t0: f32 = (aabb.min[axis] - ray_orig[axis]) * inv_d;
        var t1: f32 = (aabb.max[axis] - ray_orig[axis]) * inv_d;

        if inv_d < 0 {
            let tmp = t0;
            t0 = t1;
            t1 = tmp;
        }

        interval.start = max(t0, interval.start);
        interval.end = min(t1, interval.end);

        if interval.start >= interval.end {
            return Interval(0.0, 0.0);
        }
    }

    return interval;
}

fn get_weight(pos: vec3f) -> f32 {
    let pos3u = vec3u(u32(floor(pos.x)), u32(floor(pos.y)), u32(floor(pos.z)));
    let size = volume_grid.size.xyz;

    if any(pos3u < vec3u(0u) || pos3u >= size) {
        return 0.0;
    }

    let linear_index = pos3u.z + pos3u.y * size.z + pos3u.x * size.z * size.y;
    let block_index = linear_index / 4u;
    let num_index = linear_index % 4u;
    let current_block = weights[block_index];

    let weigth = (current_block >> (24u - num_index * 8u) & 255u);

    return f32(weigth) / f32(255u);
}

fn get_color(ray: Ray) -> RayRecord {
    let interval = hit_aabb(volume_grid.bbox, ray);
    if interval.start >= interval.end {
        return RayRecord(1.0, vec3f(0.0));
    }

    var step_size = uniforms.step_size;
    var sigma = uniforms.scattering + uniforms.absorption;

    var transparency = 1.0;
    var result = vec3f(0.0);
    let ns = u32(floor(((interval.end - interval.start) / step_size) + 0.5));

    for (var n = 0u; n < ns; n++) {
        if transparency <= 0.005 {
            break;
        }

        let t = interval.start + step_size * (f32(n) + 0.5);
        let sample_pos = ray_at(ray, t);
        var sample_weight = 0.0;
        
        sample_weight = get_weight(sample_pos);
        
        

        if sample_weight > 0.0 {
            let sample_transparency = exp(-step_size * sample_weight * (sigma));
            transparency *= sample_transparency;

            //light            
            let ray_light = create_ray(sample_pos, uniforms.light_dir.xyz);
            let interval_light = hit_aabb(volume_grid.bbox, ray_light);
            if interval_light.start < interval_light.end {
                let ns_light = u32(floor((interval_light.end / step_size) + 0.5));

                var density_light = 0.0;

                for (var nl = 0u; nl < ns_light; nl++) {
                    let t_light = min(f32(nl) * step_size, interval_light.end);
                    let sample_pos_light = ray_at(ray_light, t_light);
                    let sample_weight_light = get_weight(sample_pos_light);
                    density_light += sample_weight_light;
                }

                let light_ray_attenutation = exp(-density_light * step_size * sigma);
                let cos_theta = dot(ray.direction, -uniforms.light_dir) / (length(ray.direction) * length(uniforms.light_dir));
                result += uniforms.light_col.xyz * light_ray_attenutation * uniforms.scattering * transparency * step_size * sample_weight * phase(cos_theta);
            }
        }
    }
    return RayRecord(transparency, result);
}

fn phase(cos_theta: f32) -> f32 {
    let g = uniforms.g;
    let denom = 1.0 + g * g - 2.0 * g * cos_theta;
    return 1.0 / (4.0 / PI) * (1.0 - g * g) / (denom * sqrt(denom));
}

fn get_ray(u: f32, v: f32) -> Ray {
    return Ray(uniforms.camera_to_world * vec4f(0.0, 0.0, 0.0, 1.0),
        normalize(uniforms.camera_to_world * vec4f((u * 2.0 - 1.0) * ratio, -(v * 2.0 - 1.0), 1.0, 0.0)));
}

@compute
@workgroup_size(16,16)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    let u = f32(global_id.x) / width;
    let v = f32(global_id.y) / height;
    let ray = get_ray(u, v);
    let rec = get_color(ray);
    
    textureStore(output_texture, global_id.xy, uniforms.color * rec.transparency + vec4f(rec.color, 1.0));
}
