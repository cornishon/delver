use bracket_lib::prelude::*;

use crate::grid::Grid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Wall,
    Floor,
}

#[derive(Debug)]
pub struct Map {
    pub tiles: Grid<TileType>,
    pub revealed_tiles: Grid<bool>,
    pub visible_tiles: Grid<bool>,
    pub rooms: Vec<Rect>,
    pub width: usize,
    pub height: usize,
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles.storage[idx] == TileType::Wall
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut map = Map {
            tiles: Grid::new(TileType::Wall, width, height),
            revealed_tiles: Grid::default(width, height),
            visible_tiles: Grid::default(width, height),
            rooms: Vec::new(),
            width,
            height,
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();
        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width as i32 - w - 1) - 1;
            let y = rng.roll_dice(1, map.height as i32 - h - 1) - 1;

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

    pub fn draw(&self, draw_batch: &mut DrawBatch) {
        for (idx, tile) in self
            .tiles
            .storage
            .iter()
            .enumerate()
            .filter(|&(idx, _)| self.revealed_tiles.storage[idx])
        {
            let (fg, glyph) = match tile {
                TileType::Wall => (RGBA::from_f32(0.0, 8.0, 0.0, 1.0), '#'),
                TileType::Floor => (RGBA::from_f32(0.6, 0.5, 0.1, 1.0), '.'),
            };

            draw_batch.set(
                self.tiles.to_pos(idx),
                ColorPair {
                    fg: if self.visible_tiles.storage[idx] { fg } else { RGBA::named(GREY40) },
                    bg: RGBA::named(BLACK),
                },
                glyph,
            );
        }
    }

    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        if let Some(idx) = self.tiles.try_to_idx(x, y) {
            !self.is_opaque(idx)
        } else {
            false
        }
    }

    fn carve_room(&mut self, room: Rect) {
        for y in room.y1..=room.y2 {
            for x in room.x1..=room.x2 {
                self.tiles.set(x, y, TileType::Floor)
            }
        }
    }

    fn carve_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in x1.min(x2)..=x1.max(x2) {
            self.tiles.set(x, y, TileType::Floor)
        }
    }

    fn carve_vertical_tunnel(&mut self, x: i32, y1: i32, y2: i32) {
        for y in y1.min(y2)..=y1.max(y2) {
            self.tiles.set(x, y, TileType::Floor)
        }
    }
}
