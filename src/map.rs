use bracket_lib::prelude::*;
use grid::{Grid, Order};
use hecs::Entity;

use crate::position::Position;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    #[default]
    Wall,
    Floor,
}

#[derive(Debug)]
pub struct Map {
    pub tiles: Grid<TileType>,
    pub revealed: Grid<bool>,
    pub visible: Grid<bool>,
    pub blocked: Grid<bool>,
    pub entities: Grid<Vec<Entity>>,
    pub rooms: Vec<Rect>,
    pub width: usize,
    pub height: usize,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            tiles: Grid::new(0, 0),
            revealed: Grid::new(0, 0),
            visible: Grid::new(0, 0),
            blocked: Grid::new(0, 0),
            rooms: Default::default(),
            entities: Grid::new(0, 0),
            width: 0,
            height: 0,
        }
    }
}

impl Map {
    fn empty(width: usize, height: usize) -> Self {
        let mut map = Map {
            width,
            height,
            ..Default::default()
        };
        map.tiles = map.new_grid();
        map.revealed = map.new_grid();
        map.visible = map.new_grid();
        map.blocked = map.new_grid();
        map.entities = map.new_grid();
        map
    }

    pub fn new(width: usize, height: usize) -> Self {
        let mut map = Map::empty(width, height);

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

        for (idx, blocked) in map.blocked.indexed_iter_mut() {
            *blocked = map.tiles[idx] == TileType::Wall;
        }

        map
    }

    pub fn draw(&self, draw_batch: &mut DrawBatch) {
        for (idx @ (x, y), tile) in self
            .tiles
            .indexed_iter()
            .filter(|&(idx, _)| self.revealed[idx])
        {
            let (fg, glyph) = match tile {
                TileType::Wall => (RGBA::from_f32(0.0, 8.0, 0.0, 1.0), '#'),
                TileType::Floor => (RGBA::from_f32(0.6, 0.5, 0.1, 1.0), '.'),
            };

            draw_batch.set(
                Point::new(x, y),
                ColorPair {
                    fg: if self.visible[idx] { fg } else { RGBA::named(GREY40) },
                    bg: if self.blocked[idx] { RGBA::named(ORANGE) } else { RGBA::named(BLACK) },
                },
                glyph,
            );
        }
    }

    pub fn is_passable(&self, p: Point) -> bool {
        if self.in_bounds(p) {
            let idx2d = Position::from_point(p).into();
            let idx = self.point2d_to_index(p);
            !self.is_opaque(idx) && !self.blocked[idx2d]
        } else {
            false
        }
    }

    pub fn new_grid<T: Default>(&self) -> Grid<T> {
        Grid::new_with_order(self.width, self.height, Order::ColumnMajor)
    }

    pub fn index_to_position(&self, idx: usize) -> Position {
        Position::from_point(self.index_to_point2d(idx))
    }

    pub fn clear_entities(&mut self) {
        for content in self.entities.iter_mut() {
            content.clear();
        }
    }

    fn carve_room(&mut self, room: Rect) {
        for y in room.y1..=room.y2 {
            for x in room.x1..=room.x2 {
                if let Some(tile) = self.tiles.get_mut(x, y) {
                    *tile = TileType::Floor
                }
            }
        }
    }

    fn carve_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in x1.min(x2)..=x1.max(x2) {
            if let Some(tile) = self.tiles.get_mut(x, y) {
                *tile = TileType::Floor
            }
        }
    }

    fn carve_vertical_tunnel(&mut self, x: i32, y1: i32, y2: i32) {
        for y in y1.min(y2)..=y1.max(y2) {
            if let Some(tile) = self.tiles.get_mut(x, y) {
                *tile = TileType::Floor
            }
        }
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles.flatten()[idx] == TileType::Wall
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let p1 = self.index_to_point2d(idx1);
        let p2 = self.index_to_point2d(idx2);
        DistanceAlg::Manhattan.distance2d(p1, p2)
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let pt = self.index_to_point2d(idx);
        [
            Point::new(pt.x - 1, pt.y),
            Point::new(pt.x + 1, pt.y),
            Point::new(pt.x, pt.y + 1),
            Point::new(pt.x, pt.y - 1),
        ]
        .into_iter()
        .filter(|&p| self.is_passable(p))
        .map(|p| (self.point2d_to_index(p), 1.0))
        .collect()
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}
