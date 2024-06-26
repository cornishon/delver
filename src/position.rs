use std::ops;

use bracket_lib::terminal::Point;

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new<T>(x: T, y: T) -> Self
    where
        T: TryInto<u16> + std::fmt::Display + Copy,
    {
        Self {
            x: x.try_into()
                .unwrap_or_else(|_| panic!("{x} is out of range")),
            y: y.try_into()
                .unwrap_or_else(|_| panic!("{y} is out of range")),
        }
    }

    pub fn from_point(p: Point) -> Self {
        Self::new(p.x, p.y)
    }
}

impl ops::Add<Point> for Position {
    type Output = Position;
    fn add(self, d: Point) -> Self::Output {
        let np = Point::from(self) + d;
        Position {
            x: np.x.try_into().unwrap_or_default(),
            y: np.y.try_into().unwrap_or_default(),
        }
    }
}
impl From<&Position> for (usize, usize) {
    fn from(&Position { x, y }: &Position) -> Self {
        (x.into(), y.into())
    }
}
impl From<&mut Position> for (usize, usize) {
    fn from(&mut Position { x, y }: &mut Position) -> Self {
        (x.into(), y.into())
    }
}
impl From<Position> for (usize, usize) {
    fn from(Position { x, y }: Position) -> Self {
        (x.into(), y.into())
    }
}
impl From<&Position> for Point {
    fn from(&Position { x, y }: &Position) -> Self {
        Point::new(x, y)
    }
}
impl From<&mut Position> for Point {
    fn from(&mut Position { x, y }: &mut Position) -> Self {
        Point::new(x, y)
    }
}
impl From<Position> for Point {
    fn from(Position { x, y }: Position) -> Self {
        Point::new(x, y)
    }
}
impl TryFrom<&Point> for Position {
    type Error = <i32 as TryInto<u16>>::Error;
    fn try_from(&Point { x, y }: &Point) -> Result<Self, Self::Error> {
        Ok(Position {
            x: x.try_into()?,
            y: y.try_into()?,
        })
    }
}
impl TryFrom<Point> for Position {
    type Error = <i32 as TryInto<u16>>::Error;
    fn try_from(Point { x, y }: Point) -> Result<Self, Self::Error> {
        Ok(Position {
            x: x.try_into()?,
            y: y.try_into()?,
        })
    }
}
