use std::num::NonZeroU32;

use bevy::{
    app::{App, Plugin},
    ecs::resource::Resource,
};
use bytemuck::{Pod, Zeroable};

pub struct ChunksPlugin;

impl Plugin for ChunksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Chunks>();
    }
}

#[derive(Default, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct Chunk {
    // Some(index) where `(index >> 31) == 1` is a block id
    links: [Option<NonZeroU32>; 16],
}

#[derive(Resource)]
pub struct Chunks {
    chunks: Vec<Chunk>,
    free_chunks: Vec<NonZeroU32>,
    root: Option<NonZeroU32>,
}

impl Default for Chunks {
    fn default() -> Self {
        Self {
            chunks: vec![Chunk::default()],
            free_chunks: vec![],
            root: None,
        }
    }
}

const TOP_BIT: u32 = 1 << 31;

impl Chunks {
    pub fn set_block(&mut self, x: u32, y: u32, z: u32, w: u32, block_id: u32) {
        debug_assert_ne!(block_id & TOP_BIT, 0);
        let mut node = match self.root {
            Some(id) => id.get(),
            None => {
                let id = self.allocate_chunk();
                self.root = Some(id);
                id.get()
            }
        };
        for i in 0..31 {
            if node & TOP_BIT != 0 {
                break;
            }
            let x_bit = (x as usize >> (31 - i)) & 1;
            let y_bit = (y as usize >> (31 - i)) & 1;
            let z_bit = (z as usize >> (31 - i)) & 1;
            let w_bit = (w as usize >> (31 - i)) & 1;
            let link_index = (x_bit << 0) | (y_bit << 1) | (z_bit << 2) | (w_bit << 3);
            node = match self.chunks[node as usize].links[link_index] {
                Some(node) => node.get(),
                None => {
                    let id = self.allocate_chunk();
                    self.chunks[node as usize].links[link_index] = Some(id);
                    id.get()
                }
            };
        }
        let x_bit = x as usize & 1;
        let y_bit = y as usize & 1;
        let z_bit = z as usize & 1;
        let w_bit = w as usize & 1;
        let link_index = (x_bit << 0) | (y_bit << 1) | (z_bit << 2) | (w_bit << 3);
        self.chunks[node as usize].links[link_index] =
            Some(NonZeroU32::new(block_id | TOP_BIT).unwrap());
    }

    pub fn get_block(&self, x: u32, y: u32, z: u32, w: u32) -> Option<u32> {
        let mut node = self.root?.get();
        for i in 0..32 {
            if node & TOP_BIT != 0 {
                break;
            }
            let x_bit = (x as usize >> (31 - i)) & 1;
            let y_bit = (y as usize >> (31 - i)) & 1;
            let z_bit = (z as usize >> (31 - i)) & 1;
            let w_bit = (w as usize >> (31 - i)) & 1;
            let link_index = (x_bit << 0) | (y_bit << 1) | (z_bit << 2) | (w_bit << 3);
            node = self.chunks[node as usize].links[link_index]?.get();
        }
        debug_assert_ne!(node & TOP_BIT, 0);
        Some(node & !TOP_BIT)
    }

    fn allocate_chunk(&mut self) -> NonZeroU32 {
        match self.free_chunks.pop() {
            Some(id) => {
                self.chunks[id.get() as usize] = Chunk::default();
                id
            }
            None => {
                assert!(self.chunks.len() < TOP_BIT as usize);
                let id = self.chunks.len() as u32;
                self.chunks.push(Chunk::default());
                NonZeroU32::new(id).unwrap()
            }
        }
    }
}
