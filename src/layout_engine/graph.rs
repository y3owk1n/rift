use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn orientation(self) -> Orientation {
        match self {
            Direction::Left | Direction::Right => Orientation::Horizontal,
            Direction::Up | Direction::Down => Orientation::Vertical,
        }
    }

    pub fn opposite(self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

impl From<String> for Direction {
    fn from(s: String) -> Self {
        match s.as_str() {
            "left" => Direction::Left,
            "right" => Direction::Right,
            "up" => Direction::Up,
            "down" => Direction::Down,
            _ => panic!("Invalid direction string: {}", s),
        }
    }
}

impl Direction {
    pub fn step(&self, i: usize, len: usize) -> usize {
        match *self {
            Direction::Left => (i + len - 1) % len,
            Direction::Right => (i + 1) % len,
            _ => 0,
        }
    }
}

#[allow(unused)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutKind {
    #[default]
    Horizontal,
    Vertical,
    HorizontalStack,
    VerticalStack,
}

impl LayoutKind {
    pub fn from(orientation: Orientation) -> Self {
        match orientation {
            Orientation::Horizontal => LayoutKind::Horizontal,
            Orientation::Vertical => LayoutKind::Vertical,
        }
    }

    pub fn stack_with_offset(orientation: Orientation) -> Self {
        match orientation {
            Orientation::Horizontal => LayoutKind::HorizontalStack,
            Orientation::Vertical => LayoutKind::VerticalStack,
        }
    }

    pub fn is_stacked(self) -> bool {
        matches!(self, LayoutKind::HorizontalStack | LayoutKind::VerticalStack)
    }

    pub fn orientation(self) -> Orientation {
        use LayoutKind::*;
        match self {
            Horizontal => Orientation::Horizontal,
            Vertical => Orientation::Vertical,
            HorizontalStack => Orientation::Horizontal,
            VerticalStack => Orientation::Vertical,
        }
    }

    pub fn is_group(self) -> bool {
        matches!(self, LayoutKind::HorizontalStack | LayoutKind::VerticalStack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod direction_operations {
        use super::*;

        #[test]
        fn direction_step() {
            assert!(Direction::Right.step(0, 5) < 5);
            assert!(Direction::Left.step(4, 5) < 5);
            assert!(Direction::Up.step(0, 5) < 5);
            assert!(Direction::Down.step(4, 5) < 5);
        }

        #[test]
        fn direction_orientation() {
            assert_eq!(Direction::Left.orientation(), Orientation::Horizontal);
            assert_eq!(Direction::Right.orientation(), Orientation::Horizontal);
            assert_eq!(Direction::Up.orientation(), Orientation::Vertical);
            assert_eq!(Direction::Down.orientation(), Orientation::Vertical);
        }

        #[test]
        fn direction_opposite() {
            assert_eq!(Direction::Left.opposite(), Direction::Right);
            assert_eq!(Direction::Right.opposite(), Direction::Left);
            assert_eq!(Direction::Up.opposite(), Direction::Down);
            assert_eq!(Direction::Down.opposite(), Direction::Up);
        }
    }

    mod layout_kind_operations {
        use super::*;

        #[test]
        fn layout_kind_orientation() {
            assert_eq!(LayoutKind::Horizontal.orientation(), Orientation::Horizontal);
            assert_eq!(LayoutKind::Vertical.orientation(), Orientation::Vertical);
            assert_eq!(
                LayoutKind::HorizontalStack.orientation(),
                Orientation::Horizontal
            );
            assert_eq!(LayoutKind::VerticalStack.orientation(), Orientation::Vertical);
        }

        #[test]
        fn layout_kind_is_stacked() {
            assert!(!LayoutKind::Horizontal.is_stacked());
            assert!(!LayoutKind::Vertical.is_stacked());
            assert!(LayoutKind::HorizontalStack.is_stacked());
            assert!(LayoutKind::VerticalStack.is_stacked());
        }

        #[test]
        fn layout_kind_is_group() {
            assert!(!LayoutKind::Horizontal.is_group());
            assert!(!LayoutKind::Vertical.is_group());
            assert!(LayoutKind::HorizontalStack.is_group());
            assert!(LayoutKind::VerticalStack.is_group());
        }
    }
}
