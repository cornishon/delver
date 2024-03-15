use bracket_lib::prelude::*;

use super::CONSOLE_HEIGHT;
use super::CONSOLE_WIDTH;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TileType {
    Wall,
    Floor,
}

#[derive(Debug, Default)]
pub(crate) struct Map {
    pub(crate) tiles: Vec<TileType>,
    pub(crate) rooms: Vec<Rect>,
}

impl Map {
    pub(crate) fn new() -> Self {
        let mut map = Map {
            tiles: vec![TileType::Wall; CONSOLE_WIDTH as usize * CONSOLE_HEIGHT as usize],
            rooms: Vec::new(),
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();
        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, CONSOLE_WIDTH - w - 1) - 1;
            let y = rng.roll_dice(1, CONSOLE_HEIGHT - h - 1) - 1;

            let new_room = Rect::with_size(x, y, w, h);
            if !map.rooms.iter().any(|room| room.intersect(&new_room)) {
                map.carve_room(new_room);
                if let Some(prev_room) = map.rooms.last() {
                    let new = new_room.center();
                    let prev = prev_room.center();
                    if rng.rand() {
                        map.carve_horizontal_tunnel(prev.x, new.x, prev.y);
                        map.carve_vertical_tunnel(new.x, prev.y, new.y);
                    } else {
                        map.carve_vertical_tunnel(prev.x, prev.y, new.y);
                        map.carve_horizontal_tunnel(prev.x, new.x, new.y);
                    }
                }
                map.rooms.push(new_room);
            }
        }

        map
    }

    pub(crate) fn draw(&self, draw_batch: &mut DrawBatch) {
        let mut x = 0;
        let mut y = 0;
        for tile in &self.tiles {
            match tile {
                TileType::Wall => {
                    draw_batch.set(
                        Point { x, y },
                        ColorPair::new(RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0., 0., 0.)),
                        '#',
                    );
                }
                TileType::Floor => {
                    draw_batch.set(
                        Point { x, y },
                        ColorPair::new(RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.)),
                        '.',
                    );
                }
            }
            x += 1;
            if x >= CONSOLE_WIDTH {
                x = 0;
                y += 1;
            }
        }
    }

    fn carve_room(&mut self, room: Rect) {
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                self.tiles[xy_idx(x, y)] = TileType::Floor;
            }
        }
    }

    fn carve_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in x1.min(x2)..=x1.max(x2) {
            if let Some(tile) = self.tiles.get_mut(xy_idx(x, y)) {
                *tile = TileType::Floor;
            }
        }
    }

    fn carve_vertical_tunnel(&mut self, x: i32, y1: i32, y2: i32) {
        for y in y1.min(y2)..=y1.max(y2) {
            if let Some(tile) = self.tiles.get_mut(xy_idx(x, y)) {
                *tile = TileType::Floor;
            }
        }
    }
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * CONSOLE_WIDTH as usize) + x as usize
}
