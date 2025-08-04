use specs::{System, Write, Read, ReadStorage, Join};
use crate::{
    resources::{RenderData, Vertex, ScreenDimensions},
    components::{Position, Renderable},
};

pub struct RenderingSystem;

impl<'a> System<'a> for RenderingSystem {
    type SystemData = (
        Write<'a, RenderData>,
        Read<'a, ScreenDimensions>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Renderable>,
    );

    fn run(&mut self, (mut render_data, screen_dim, positions, renderables): Self::SystemData) {
        render_data.0.clear();

        for (pos, render) in (&positions, &renderables).join() {
            // This logic converts world coordinates (e.g., -500 to 500)
            // into clip-space coordinates (-1.0 to 1.0) that the GPU expects.
            let half_w = render.width / screen_dim.width;
            let half_h = render.height / screen_dim.height;
            let center_x = (pos.0.x / screen_dim.width) * 2.0;
            let center_y = (pos.0.y / screen_dim.height) * 2.0;

            let x_min = center_x - half_w;
            let x_max = center_x + half_w;
            let y_min = center_y - half_h;
            let y_max = center_y + half_h;

            let vertices = [
                Vertex { position: [x_min, y_min], color: render.color },
                Vertex { position: [x_max, y_min], color: render.color },
                Vertex { position: [x_max, y_max], color: render.color },
                Vertex { position: [x_min, y_min], color: render.color },
                Vertex { position: [x_max, y_max], color: render.color },
                Vertex { position: [x_min, y_max], color: render.color },
            ];
            render_data.0.extend_from_slice(&vertices);
        }
    }
}
