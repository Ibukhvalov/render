pub fn min(a: &f32, b: &f32) -> f32 {
    if a<b { return *a; }
    *b
}

pub fn max(a: &f32, b: &f32) -> f32 {
    if a>b { return *a; }
    *b
}