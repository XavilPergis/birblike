use cgmath::Vector2;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Tile {
    Border(BorderTile),
    Air,
}

impl Tile {
    pub fn sprite(&self) -> Vector2<f32> {
        match *self {
            Tile::Air => Vector2::new(0.0, 0.0),
            Tile::Border(border_tile) => border_tile.sprite(),
        }
    }

    /// Gets the associated BorderTile if it is present; None otherwise
    pub fn as_border(&self) -> Option<BorderTile> {
        if let Tile::Border(bt) = *self { Some(bt) } else { None }
    }
}

pub const SPRITE_AIR: Vector2<f32> = Vector2 { x: 0.0, y: 0.0 };

pub const SPRITE_VERTICAL_BORDER: Vector2<f32> = Vector2 { x: 10.0, y: 11.0 };
pub const SPRITE_HORIZONTAL_BORDER: Vector2<f32> = Vector2 { x: 13.0, y: 12.0 };
pub const SPRITE_CROSS_BORDER: Vector2<f32> = Vector2 { x: 14.0, y: 12.0 };

pub const SPRITE_BEND_TOP_LEFT: Vector2<f32> = Vector2 { x: 9.0, y: 12.0 };
pub const SPRITE_BEND_TOP_RIGHT: Vector2<f32> = Vector2 { x: 11.0, y: 11.0 };
pub const SPRITE_BEND_BOTTOM_LEFT: Vector2<f32> = Vector2 { x: 8.0, y: 12.0 };
pub const SPRITE_BEND_BOTTOM_RIGHT: Vector2<f32> = Vector2 { x: 12.0, y: 11.0 };

pub const SPRITE_T_POINT_UP: Vector2<f32> = Vector2 { x: 10.0, y: 12.0 };
pub const SPRITE_T_POINT_DOWN: Vector2<f32> = Vector2 { x: 11.0, y: 12.0 };
pub const SPRITE_T_POINT_LEFT: Vector2<f32> = Vector2 { x: 9.0, y: 11.0 };
pub const SPRITE_T_POINT_RIGHT: Vector2<f32> = Vector2 { x: 12.0, y: 12.0 };

pub const SPRITE_NUB_TOP: Vector2<f32> = Vector2 { x: 6.0, y: 13.0 };
pub const SPRITE_NUB_BOTTOM: Vector2<f32> = Vector2 { x: 13.0, y: 11.0 };
pub const SPRITE_NUB_LEFT: Vector2<f32> = Vector2 { x: 5.0, y: 13.0 };
pub const SPRITE_NUB_RIGHT: Vector2<f32> = Vector2 { x: 14.0, y: 11.0 };

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct BorderTile {
    top: bool,
    bottom: bool,
    left: bool,
    right: bool,
}

impl BorderTile {
    pub fn empty() -> Self {
        BorderTile { top: false, bottom: false, left: false, right: false }
    }

    pub fn sprite(&self) -> Vector2<f32> {
        match (self.top, self.bottom, self.left, self.right) {
            (true,  false, true,  true ) => SPRITE_T_POINT_UP,
            (false, true,  true,  true ) => SPRITE_T_POINT_DOWN,
            (true,  true,  true,  false) => SPRITE_T_POINT_LEFT,
            (true,  true,  false, true ) => SPRITE_T_POINT_RIGHT,
            
            (true,  false, false, false) => SPRITE_NUB_TOP,
            (false, true,  false, false) => SPRITE_NUB_BOTTOM,
            (false, false, true,  false) => SPRITE_NUB_LEFT,
            (false, false, false, true ) => SPRITE_NUB_RIGHT,

            (false, true,  false, true ) => SPRITE_BEND_TOP_LEFT,
            (true,  false, false, true ) => SPRITE_BEND_BOTTOM_LEFT,
            (false, true,  true,  false) => SPRITE_BEND_TOP_RIGHT,
            (true,  false, true,  false) => SPRITE_BEND_BOTTOM_RIGHT,

            (true,  true,  false, false) => SPRITE_HORIZONTAL_BORDER,
            (false, false, true,  true ) => SPRITE_VERTICAL_BORDER,

            (true,  true,  true,  true ) => SPRITE_CROSS_BORDER,
            (false, false, false, false) => SPRITE_AIR,
        }
    }
}

pub const BORDER_T_POINT_UP: BorderTile        = BorderTile { top: true,  bottom: false, left: true,  right: true  };
pub const BORDER_T_POINT_DOWN: BorderTile      = BorderTile { top: false, bottom: true,  left: true,  right: true  };
pub const BORDER_T_POINT_LEFT: BorderTile      = BorderTile { top: true,  bottom: true,  left: true,  right: false };
pub const BORDER_T_POINT_RIGHT: BorderTile     = BorderTile { top: true,  bottom: true,  left: false, right: true  };
pub const BORDER_NUB_TOP: BorderTile           = BorderTile { top: true,  bottom: false, left: false, right: false };
pub const BORDER_NUB_BOTTOM: BorderTile        = BorderTile { top: false, bottom: true,  left: false, right: false };
pub const BORDER_NUB_LEFT: BorderTile          = BorderTile { top: false, bottom: false, left: true,  right: false };
pub const BORDER_NUB_RIGHT: BorderTile         = BorderTile { top: false, bottom: false, left: false, right: true  };
pub const BORDER_BEND_TOP_LEFT: BorderTile     = BorderTile { top: false, bottom: true,  left: false, right: true  };
pub const BORDER_BEND_BOTTOM_LEFT: BorderTile  = BorderTile { top: true,  bottom: false, left: false, right: true  };
pub const BORDER_BEND_TOP_RIGHT: BorderTile    = BorderTile { top: false, bottom: true,  left: true,  right: false };
pub const BORDER_BEND_BOTTOM_RIGHT: BorderTile = BorderTile { top: true,  bottom: false, left: true,  right: false };
pub const BORDER_HORIZONTAL: BorderTile        = BorderTile { top: false, bottom: false, left: true,  right: true  };
pub const BORDER_VERTICAL: BorderTile          = BorderTile { top: true,  bottom: true,  left: false, right: false };
pub const BORDER_CROSS: BorderTile             = BorderTile { top: true,  bottom: true,  left: true,  right: true  };
pub const BORDER_AIR: BorderTile               = BorderTile { top: false, bottom: false, left: false, right: false };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_borders() {
        assert_eq!(BORDER_BEND_TOP_LEFT | BORDER_HORIZONTAL, BORDER_T_POINT_DOWN);
        assert_eq!(BORDER_BEND_TOP_RIGHT | BORDER_HORIZONTAL, BORDER_T_POINT_DOWN);
        assert_eq!(BORDER_BEND_BOTTOM_LEFT | BORDER_HORIZONTAL, BORDER_T_POINT_UP);
        assert_eq!(BORDER_BEND_BOTTOM_RIGHT | BORDER_HORIZONTAL, BORDER_T_POINT_UP);
        assert_eq!(BORDER_BEND_TOP_LEFT | BORDER_VERTICAL, BORDER_T_POINT_RIGHT);
        assert_eq!(BORDER_BEND_TOP_RIGHT | BORDER_VERTICAL, BORDER_T_POINT_LEFT);
        assert_eq!(BORDER_BEND_BOTTOM_LEFT | BORDER_VERTICAL, BORDER_T_POINT_RIGHT);
        assert_eq!(BORDER_BEND_BOTTOM_RIGHT | BORDER_VERTICAL, BORDER_T_POINT_LEFT);
    }
}

impl ::std::ops::BitOr for BorderTile {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        BorderTile {
            top: self.top | rhs.top,
            bottom: self.bottom | rhs.bottom,
            left: self.left | rhs.left,
            right: self.right | rhs.right,
        }
    }
}

impl ::std::ops::BitOrAssign for BorderTile {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

