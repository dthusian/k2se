use std::collections::HashMap;
use crate::layout::shapers::LayoutShaper;

const CHUNK_SZ: usize = 16;
const CHUNK_SZI: i32 = CHUNK_SZ as i32;

struct Chunk {
  x2_free: Vec<(i32, i32)>,
  x1_free: Vec<(i32, i32)>
}

/// Holds a set of free locations and performs searches for free space on it.
/// Optimized for searching for 2x1-wide slots because that's how large combinators are
pub struct LocationSearcher {
  shaper: Box<dyn LayoutShaper>,
  chunks: HashMap<(i32, i32), Chunk>
}

impl LocationSearcher {
  pub fn new(layout_shaper: Box<dyn LayoutShaper>) -> Self {
    LocationSearcher {
      shaper: layout_shaper,
      chunks: Default::default(),
    }
  }
  
  fn gen_chunk(&mut self, coord: (i32, i32)) {
    let bitmap = self.shaper.is_free_area(coord, (coord.0 + CHUNK_SZI, coord.1 + CHUNK_SZI));
    let mut chunk = Chunk {
      x2_free: vec![],
      x1_free: vec![],
    };
    // x2 search
    for i in 0..CHUNK_SZ {
      for j in 0..CHUNK_SZ / 2 {
        if bitmap[i][j * 2] && bitmap[i][j * 2 + 1] {
          chunk.x2_free.push((i as i32, j as i32 * 2));
          bitmap[i][j * 2] = false;
          bitmap[i][j * 2 + 1] = false;
        }
      }
    }
    // x1 search
    for i in 0..CHUNK_SZ {
      for j in 0..CHUNK_SZ {
        if bitmap[i][j] {
          chunk.x1_free.push((i as i32, j as i32));
          bitmap[i][j] = false;
        }
      }
    }
    self.chunks.insert(coord, chunk);
  }
  
  fn ensure_chunk(&mut self, coord: (i32, i32)) {
    if !self.chunks.contains_key(&coord) {
      self.gen_chunk(coord);
    }
  }
  
  pub fn take_nearest_x2(&mut self, x: (i32, i32), max_dist: f64) -> Option<(i32, i32)> {
    let chunk_coord = (CHUNK_SZI * (x.0 / CHUNK_SZI), CHUNK_SZI * (x.1 / CHUNK_SZI));
    let chunk_coord_list = (-1..=1).map(|i| (-1..=1).map(|j| (i, j)))
      .flatten()
      .map(|v| {
        (chunk_coord.0 + v.0 * CHUNK_SZI, chunk_coord.1 + v.1 * CHUNK_SZI)
      })
      .collect::<Vec<_>>();
    todo!()
  }
}