pub trait GetRatio {
    fn get_ratio(&self) -> f32;
}

impl GetRatio for [u32; 2] {
    fn get_ratio(&self) -> f32 {
        self[0] as f32 / self[1] as f32
    }
}

#[inline]
pub fn spread(elem_count: u8, pos: u8) -> f32 {
    ((pos + 1) as f32 / (elem_count + 1) as f32) * 2.0 - 1.0
}
