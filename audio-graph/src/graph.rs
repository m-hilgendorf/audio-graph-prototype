#![allow(dead_code, unused_variables)]
pub enum Error {

}

type Result<T> = std::result::Result<T, Error>;

pub struct AudioNode {

}

pub struct AudioGraph {

}

impl AudioGraph {
    pub fn new() -> Self { todo!() }
    pub fn create_node(&mut self, node_name:impl AsRef<str>) -> Result<AudioNode> { todo!() }
    pub fn create_port(&mut self, node:AudioNode, port_name:impl AsRef<str>) -> Result<()> { todo!() }
    pub fn remove_node(&mut self, node:AudioNode) -> Result<()> { todo!() }
    pub fn remove_port(&mut self, node:AudioNode, port_name: impl AsRef<str>) -> Result<()> { todo!() }
    pub fn update_delay(&mut self, node:AudioNode, delay:f64) -> Result<()> { todo!() }
}
