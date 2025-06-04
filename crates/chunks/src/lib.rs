use arrayvec::ArrayVec;
use bevy::{
    app::{App, Plugin},
    ecs::resource::Resource,
};
use bytemuck::{Pod, Zeroable};
use std::collections::BTreeSet;

pub struct ChunksPlugin;

impl Plugin for ChunksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Chunks>();
    }
}

const BLOCK_BIT: u32 = 1 << 31;

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct Chunk {
    // `(index >> 31) == 1` is a block id
    links: [u32; 16],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            links: [BLOCK_BIT; 16],
        }
    }
}

#[derive(Resource, Debug)]
pub struct Chunks {
    chunks: Vec<Chunk>,
    free_chunks: BTreeSet<u32>,
    root: u32,
}

impl Default for Chunks {
    fn default() -> Self {
        Self {
            chunks: vec![Chunk::default()],
            free_chunks: BTreeSet::new(),
            root: 0,
        }
    }
}

impl Chunks {
    pub fn set_block(&mut self, x: u32, y: u32, z: u32, w: u32, block_id: u32) {
        debug_assert_eq!(block_id & BLOCK_BIT, 0);
        let mut nodes = ArrayVec::<_, 32>::new();
        nodes.push((self.root, 0));
        for index in 0..31 {
            let node = nodes.last().unwrap().0;
            let link_index = Self::link_index(x, y, z, w, index);
            let id = self.chunks[node as usize].links[link_index];
            nodes.push((
                if id & BLOCK_BIT != 0 {
                    let new_id = self.allocate_chunk();
                    for link in &mut self.chunks[new_id as usize].links {
                        *link = id;
                    }
                    self.chunks[node as usize].links[link_index] = new_id;
                    new_id
                } else {
                    id
                },
                link_index,
            ));
        }
        let bottom_node = nodes.last().unwrap().0;
        let link_index = Self::link_index(x, y, z, w, 31);
        self.chunks[bottom_node as usize].links[link_index] = block_id | BLOCK_BIT;
        while let Some((id, parent_link_index)) = nodes.pop() {
            let first = self.chunks[id as usize].links[0];
            if self.chunks[id as usize].links[1..]
                .iter()
                .all(|&element| first == element)
            {
                if let Some(&(parent_id, _)) = nodes.last() {
                    self.deallocate_chunk(id);
                    self.chunks[parent_id as usize].links[parent_link_index] = first;
                }
            }
        }
        self.cleanup_unused_chunks();
    }

    pub fn get_block(&self, x: u32, y: u32, z: u32, w: u32) -> u32 {
        let mut node = self.root;
        for index in 0..32 {
            if node & BLOCK_BIT != 0 {
                break;
            }
            let link_index = Self::link_index(x, y, z, w, index);
            node = self.chunks[node as usize].links[link_index];
        }
        debug_assert_ne!(node & BLOCK_BIT, 0);
        node & !BLOCK_BIT
    }

    fn link_index(x: u32, y: u32, z: u32, w: u32, index: usize) -> usize {
        let x_bit = (x as usize >> (31 - index)) & 1;
        let y_bit = (y as usize >> (31 - index)) & 1;
        let z_bit = (z as usize >> (31 - index)) & 1;
        let w_bit = (w as usize >> (31 - index)) & 1;
        (x_bit << 0) | (y_bit << 1) | (z_bit << 2) | (w_bit << 3)
    }

    fn allocate_chunk(&mut self) -> u32 {
        match self.free_chunks.pop_first() {
            Some(id) => {
                self.chunks[id as usize] = Chunk::default();
                id
            }
            None => {
                assert!(self.chunks.len() < BLOCK_BIT as usize);
                let id = self.chunks.len() as u32;
                self.chunks.push(Chunk::default());
                id
            }
        }
    }

    fn deallocate_chunk(&mut self, id: u32) {
        self.free_chunks.insert(id);
    }

    fn cleanup_unused_chunks(&mut self) {
        while let Some(&last) = self.free_chunks.last() {
            if self.chunks.len() - 1 != last as usize {
                break;
            }
            self.chunks.pop();
            println!("removed {}", self.chunks.len());
            self.free_chunks.pop_last();
        }
    }
}
