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
}

impl ops::Add<Point> for Position {
    type Output = Position;
    fn add(self, d: Point) -> Self::Output {
        (Point::from(self) + d).try_into().unwrap()
    }
}
impl From<&Position> for (usize, usize) {
    fn from(&Position { x, y }: &Position) -> Self {
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
