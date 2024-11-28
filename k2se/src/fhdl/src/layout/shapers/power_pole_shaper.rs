use std::collections::HashMap;
use crate::err::Cerr;
use crate::layout::shapers::{LayoutShaper, util_get_parse_opt, util_prop_error};

#[derive(Default)]
pub struct PowerPoleShaper {
  pub power_pole_size: u32,
  pub power_pole_range: u32,
}

impl LayoutShaper for PowerPoleShaper {
  fn is_free(&self, pos: (i32, i32)) -> bool {
    if (pos.0 as u32).rem_euclid(self.power_pole_range) < self.power_pole_size {
      false
    } else {
      true
    }
  }

  // [x][y]
  fn is_free_area(&self, first_corner: (i32, i32), second_corner: (i32, i32)) -> Vec<Vec<bool>> {
    let mut vec = vec![];
    for i in first_corner.0 .. second_corner.0 {
      vec.push(vec![]);
      for j in first_corner.1 .. second_corner.1 {
        vec[i].push(self.is_free((i, j)));
      }
    }
    vec
  }
}

fn create_power_pole_shaper(opts: &HashMap<String, String>) -> Result<Box<dyn LayoutShaper>, Cerr> {
  Ok(Box::new(PowerPoleShaper {
    power_pole_size: util_get_parse_opt::<u32>(&opts, "power_pole_size")?
      .unwrap_or(2),
    power_pole_range: util_get_parse_opt::<u32>(&opts, "power_pole_range")?
      .unwrap_or(18),
  }))
}

