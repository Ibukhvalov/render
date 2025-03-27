@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<storage, read> volume_grid: VolumeGridStatic;

@group(0) @binding(2)
var<storage, read> weights: array<u32>;

@group(0) @binding(3)
var<uniform> uniforms: Uniforms;

@group(0) @binding(4)
var<storage, read_write> acc_ctx: array<accumulation_context, num_pixels>;

const width = 800.0;
const height = 600.0;
const num_pixels = u32(width) * u32(height);

const INF = 99999.0;

const ratio = width / height;

const PI: f32 = 3.14159265358979323846;

const BASE_WEIGHT: f32 = 0.1;


struct accumulation_context {
    color: vec3f,
    transparency: f32,
    t_start: f32,
    t_end: f32,
}

struct Uniforms {
    camera_to_world: mat4x4f,
    color: vec4f,
    light_dir: vec3f,
    light_col: vec3f,
    scattering: f32,
    absorption: f32,
    g: f32,
    step_size: f32,
    sub_frame_idx: i32,
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
    let block_index = linear_index / 32u;
    let bit_index = linear_index % 32u;
    let current_block = weights[block_index];

    if(((current_block >> (31u - bit_index)) & 1u) == 1u) {
        return BASE_WEIGHT;
    }
    return 0.0;
}



fn get_color(ray: Ray, buffer_idx: u32) -> vec3f {

    var step_size = uniforms.step_size;
    var sigma = uniforms.scattering + uniforms.absorption;

    var result = vec3f(0.0);

    //if transparency <= 0.005 {
    //    break;
    //}
    let t = acc_ctx[buffer_idx].t_start + step_size * f32(uniforms.sub_frame_idx);

    if (t > acc_ctx[buffer_idx].t_end) {
        return vec3f(0.0);
    }

    let sample_pos = ray_at(ray, t);
    let sample_weight = get_weight(sample_pos);    

    if sample_weight > 0.0 {
        let sample_transparency = exp(-step_size * sample_weight * (sigma));
        acc_ctx[buffer_idx].transparency *= sample_transparency;

        //light            
        let ray_light = create_ray(sample_pos, uniforms.light_dir);
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
            let cos_theta = dot(ray.direction.xyz, -uniforms.light_dir) / (length(ray.direction) * length(uniforms.light_dir));
            result += uniforms.light_col * light_ray_attenutation * uniforms.scattering * acc_ctx[buffer_idx].transparency * step_size * sample_weight * phase(cos_theta);
        }
    }
    return result;
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


fn init_accumulation(ray: Ray, buffer_idx: u32) {
    let hit_interval = hit_aabb(volume_grid.bbox, ray);
    acc_ctx[buffer_idx].t_start = hit_interval.start;
    acc_ctx[buffer_idx].t_end = hit_interval.end;
    acc_ctx[buffer_idx].transparency = 1.0;
    acc_ctx[buffer_idx].color = vec3f(0.0);
}

@compute
@workgroup_size(16,16)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    
    if(global_id.x > u32(width) || global_id.y > u32(height)) {
        return;
    }

    let u = f32(global_id.x) / width;
    let v = f32(global_id.y) / height;

    //textureStore(output_texture, global_id.xy, vec4f(f32(uniforms.sub_frame_idx) / 1000.0));

    let buffer_idx = global_id.y*u32(width) + global_id.x;

    let ray = get_ray(u, v);

    if (uniforms.sub_frame_idx <= 0) {
        init_accumulation(ray, buffer_idx);
    } else if (acc_ctx[buffer_idx].t_start < acc_ctx[buffer_idx].t_end) {

        let col = get_color(ray, buffer_idx);
        acc_ctx[buffer_idx].color += col;
    }

    textureStore(output_texture, global_id.xy, uniforms.color * acc_ctx[buffer_idx].transparency + vec4f(acc_ctx[buffer_idx].color, 1.0));

    //textureStore(output_texture, global_id.xy, uniforms.color * acc_ctx[buffer_idx].transparency + vec4f(acc_ctx[buffer_idx].color, 1.0));

}
