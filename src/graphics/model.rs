use crate::graphics::MyVertex;

pub struct Model {
    pub vertices: Vec<MyVertex>,
    pub indices: Vec<u32>
}

pub trait IsModel {

} 