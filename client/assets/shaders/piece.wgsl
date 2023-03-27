struct PieceParams {
  sprite_origin_x: f32,
  sprite_origin_y: f32,

  // sides to blur
  // bitmask:
  // 0b0001 = north
  // 0b0010 = south
  // 0b0100 = east
  // 0b1000 = west
  open_sides: u32,

  padding: u32,
}

@group(1) @binding(0)
var<uniform> params: PieceParams;

@group(1) @binding(1)
var texture: texture_2d<f32>;

@group(1) @binding(2)
var texture_sampler: sampler;


const directions: f32 = 8.0;
const quality: f32 = 4.0;
const size: f32 = 0.0;

const pi2 = 6.28318530718;

@fragment
fn fragment(
   #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {

   let dim = vec2<f32>(textureDimensions(texture));
   let radius = size / dim;
   let color = textureSample(texture, texture_sampler, uv);
   var summed_color = color;

   for (var d = 0.0; d < pi2; d += pi2 / directions) {
       for (var i = 1.0 / quality; i <= 1.0; i += 1.0 / quality) {
           summed_color += textureSample(texture, texture_sampler, uv + vec2(cos(d), sin(d)) * radius * i);
       }
   }

   // uncomment to view uv origin (0.0, 0.0)

   if abs(uv.x - 0.5) < 0.005 || abs(uv.y - 0.5) < 0.005 {
     return vec4(1.0, 0.0, 0.0, 1.0);
   }

   // uncomment to view sprite origin

   if abs(uv.x - params.sprite_origin_x) < 0.005 || abs(uv.y - params.sprite_origin_y) < 0.005 {
     return vec4(1.0, 0.0, 0.0, 1.0);
   }

   // only blur near edges
   if summed_color.w >= 1.0 + directions * quality || summed_color.w == 0.0 {
       return color;
   }

   let blurred_color = summed_color / (directions * quality + (directions / 2.0 - 1.0));

   let rot45 = mat2x2(0.70710678118, -0.70710678118, 0.70710678118, 0.70710678118);
   let min_dim = min(dim.x, dim.y);
   let uv = uv - vec2(params.sprite_origin_x, params.sprite_origin_y);
   let hadamard = vec2(uv.x * dim.x, uv.y * dim.y);
   let uv_prime = rot45 * (hadamard / min_dim);

   // uncomment to debug edges

   if uv_prime.x < 0.0 {
      if uv_prime.y > 0.0 {
          // west
          return vec4(1.0, 0.0, 0.0, 1.0);
      } else if uv_prime.y < 0.0 {
          // north
          return vec4(0.0, 1.0, 0.0, 1.0);
      }
   } else {
      if uv_prime.y > 0.0 {
          // south
          return vec4(0.0, 0.0, 1.0, 1.0);
      } else if uv_prime.y < 0.0 {
          // east
          return vec4(1.0, 0.0, 1.0, 1.0);
      }
   }

   /*if uv_prime.x < 0.0 {*/
   /*    if uv_prime.y > 0.0 && (params.open_sides & 8u) == 8u {*/
   /*        // west*/
   /*        return blurred_color;*/
   /*    } else if uv_prime.y < 0.0 && (params.open_sides & 1u) == 1u {*/
   /*        // north*/
   /*        return blurred_color;*/
   /*    }*/
   /*} else {*/
   /*    if uv_prime.y > 0.0 && (params.open_sides & 2u) == 2u {*/
   /*        // south*/
   /*        return blurred_color;*/
   /*    } else if uv_prime.y < 0.0 && (params.open_sides & 4u) == 4u {*/
   /*        // east*/
   /*        return blurred_color;*/
   /*    }*/
   /*}*/

   /*return color;*/
}
