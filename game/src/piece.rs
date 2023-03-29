use std::rc::Rc;

use image::Pixel;
use resvg::{tiny_skia, usvg};
use serde::{Deserialize, Serialize};
use usvg::NodeExt;

use crate::Puzzle;

const TAB_LENGTH_RATIO: f64 = 0.30;
const TAB_OUTER_SIZE_RATIO: f64 = 0.36;
const TAB_INNER_SIZE_RATIO: f64 = 0.22;
const PIECE_OVERSIZE_RATIO: f64 = 0.005;
pub(crate) const BORDER_SIZE_DENOM: u32 = 10;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct PieceIndex(pub u8, pub u8);

impl PieceIndex {
    pub fn neighbors(self, puzzle_width: u8, puzzle_height: u8) -> Vec<Self> {
        [
            self.north_neighbor(),
            self.south_neighbor(puzzle_height),
            self.east_neighbor(puzzle_width),
            self.west_neighbor(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn north_neighbor(self) -> Option<Self> {
        let PieceIndex(row, col) = self;
        if row > 0 {
            return Some(PieceIndex(row - 1, col));
        }
        None
    }

    pub fn south_neighbor(self, puzzle_height: u8) -> Option<Self> {
        let PieceIndex(row, col) = self;
        if row < puzzle_height - 1 {
            return Some(PieceIndex(row + 1, col));
        }
        None
    }

    pub fn east_neighbor(self, puzzle_width: u8) -> Option<Self> {
        let PieceIndex(row, col) = self;
        if col < puzzle_width - 1 {
            return Some(PieceIndex(row, col + 1));
        }
        None
    }

    pub fn west_neighbor(self) -> Option<Self> {
        let PieceIndex(row, col) = self;
        if col > 0 {
            return Some(PieceIndex(row, col - 1));
        }
        None
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum PieceKind {
    TopLeftCorner,

    TopRightCornerEven,
    TopRightCornerOdd,

    TopEdgeEven,
    TopEdgeOdd,

    BottomLeftCornerEven,
    BottomLeftCornerOdd,

    BottomEdgeEven,
    BottomEdgeOdd,

    BottomRightCornerEven,
    BottomRightCornerOdd,

    LeftEdgeEven,
    LeftEdgeOdd,

    RightEdgeEven,
    RightEdgeOdd,

    MiddleEven,
    MiddleOdd,
}

impl PieceKind {
    pub fn new(index: PieceIndex, puzzle_width: u8, puzzle_height: u8) -> Self {
        use PieceKind::*;
        let PieceIndex(row, col) = index;
        let even = (row + col) % 2 == 0;

        #[allow(clippy::collapsible_else_if)]
        if row == 0 {
            if col == 0 {
                TopLeftCorner
            } else if col == puzzle_width - 1 {
                if even {
                    TopRightCornerEven
                } else {
                    TopRightCornerOdd
                }
            } else {
                if even {
                    TopEdgeEven
                } else {
                    TopEdgeOdd
                }
            }
        } else if row == puzzle_height - 1 {
            if col == 0 {
                if even {
                    BottomLeftCornerEven
                } else {
                    BottomLeftCornerOdd
                }
            } else if col == puzzle_width - 1 {
                if even {
                    BottomRightCornerEven
                } else {
                    BottomRightCornerOdd
                }
            } else {
                if even {
                    BottomEdgeEven
                } else {
                    BottomEdgeOdd
                }
            }
        } else {
            if col == 0 {
                if even {
                    LeftEdgeEven
                } else {
                    LeftEdgeOdd
                }
            } else if col == puzzle_width - 1 {
                if even {
                    RightEdgeEven
                } else {
                    RightEdgeOdd
                }
            } else {
                if even {
                    MiddleEven
                } else {
                    MiddleOdd
                }
            }
        }
    }

    pub(crate) fn tabs(&self) -> (u32, u32, u32, u32) {
        use PieceKind::*;

        // north south east west
        match self {
            TopLeftCorner => (0, 0, 1, 0),

            TopRightCornerEven => (0, 0, 0, 1),
            TopRightCornerOdd => (0, 1, 0, 0),

            TopEdgeEven => (0, 0, 1, 1),
            TopEdgeOdd => (0, 1, 0, 0),

            BottomLeftCornerEven => (0, 0, 1, 0),
            BottomLeftCornerOdd => (1, 0, 0, 0),

            BottomEdgeEven => (0, 0, 1, 1),
            BottomEdgeOdd => (1, 0, 0, 0),

            BottomRightCornerEven => (0, 0, 0, 1),
            BottomRightCornerOdd => (1, 0, 0, 0),

            LeftEdgeEven => (0, 0, 1, 0),
            LeftEdgeOdd => (1, 1, 0, 0),

            RightEdgeEven => (0, 0, 0, 1),
            RightEdgeOdd => (1, 1, 0, 0),

            MiddleEven => (0, 0, 1, 1),
            MiddleOdd => (1, 1, 0, 0),
        }
    }

    pub(crate) fn blanks(&self) -> (u32, u32, u32, u32) {
        use PieceKind::*;

        // north south east west
        match self {
            TopLeftCorner => (0, 1, 0, 0),

            TopRightCornerEven => (0, 1, 0, 0),
            TopRightCornerOdd => (0, 0, 0, 1),

            TopEdgeEven => (0, 1, 0, 0),
            TopEdgeOdd => (0, 0, 1, 1),

            BottomLeftCornerEven => (1, 0, 0, 0),
            BottomLeftCornerOdd => (0, 0, 1, 0),

            BottomEdgeEven => (1, 0, 0, 0),
            BottomEdgeOdd => (0, 0, 1, 1),

            BottomRightCornerEven => (1, 0, 0, 0),
            BottomRightCornerOdd => (0, 0, 0, 1),

            LeftEdgeEven => (1, 1, 0, 0),
            LeftEdgeOdd => (0, 0, 1, 0),

            RightEdgeEven => (1, 1, 0, 0),
            RightEdgeOdd => (0, 0, 0, 1),

            MiddleEven => (1, 1, 0, 0),
            MiddleOdd => (0, 0, 1, 1),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Piece {
    index: PieceIndex,
    kind: PieceKind,
    sprite: crate::image::Image,
    sprite_origin_x: f64,
    sprite_origin_y: f64,
    pub(crate) transform: bevy::prelude::Transform,
    pub(crate) group_index: usize,
}

impl Piece {
    pub fn new(
        puzzle: &Puzzle,
        index: PieceIndex,
        group_index: usize,
        image: &mut image::RgbaImage,
        border_size: u32,
    ) -> Self {
        let kind = PieceKind::new(index, puzzle.puzzle_width(), puzzle.puzzle_height());

        let (sprite, sprite_origin_x, sprite_origin_y) =
            Piece::cut_sprite(index, puzzle, border_size, image, kind);

        // TODO
        let padding = 120;
        let initial_position = bevy::prelude::Vec3::new(
            index.1 as f32 * (puzzle.piece_height() + padding) as f32,
            -(index.0 as f32 * (puzzle.piece_width() + padding) as f32),
            0.0,
        );

        Piece {
            index,
            kind,
            sprite: sprite.into(),
            sprite_origin_x,
            sprite_origin_y,
            transform: bevy::prelude::Transform::from_translation(initial_position),
            group_index,
        }
    }

    fn tab_size(piece_width: u32, piece_height: u32) -> (u32, u32) {
        (
            (TAB_LENGTH_RATIO * f64::from(piece_width)) as u32,
            (TAB_LENGTH_RATIO * f64::from(piece_height)) as u32,
        )
    }

    pub fn sprite_origin(&self, piece_width: u32, piece_height: u32) -> (u32, u32) {
        let (north_tab, _, _, west_tab) = self.kind.tabs();
        let (tab_width, tab_height) = Piece::tab_size(piece_width, piece_height);
        (tab_width * west_tab, tab_height * north_tab)
    }

    fn cut_sprite(
        index: PieceIndex,
        puzzle: &Puzzle,
        border_size: u32,
        image: &mut image::RgbaImage,
        kind: PieceKind,
    ) -> (image::RgbaImage, f64, f64) {
        let PieceIndex(row, col) = index;
        let piece_width = puzzle.piece_width();
        let piece_height = puzzle.piece_height();
        let (tab_width, tab_height) = Piece::tab_size(piece_width, piece_height);
        let (north_tab, south_tab, east_tab, west_tab) = kind.tabs();
        let (north_blank, south_blank, east_blank, west_blank) = kind.blanks();

        let sprite_width = piece_width + tab_width * (east_tab + west_tab) + 2 * border_size;
        let sprite_height = piece_height + tab_height * (north_tab + south_tab) + 2 * border_size;

        let sprite_origin_x: f64 = (border_size + piece_width / 2 + west_tab * tab_width).into();
        let sprite_origin_y: f64 = (border_size + piece_height / 2 + south_tab * tab_height).into();

        let mut crop = image::imageops::crop(
            image,
            col as u32 * piece_width - tab_width * west_tab,
            row as u32 * piece_height - tab_height * north_tab,
            sprite_width,
            sprite_height,
        )
        .to_image();

        let tree_size = usvg::Size::new(sprite_width.into(), sprite_height.into()).unwrap();
        let tree = usvg::Tree {
            size: tree_size,
            view_box: usvg::ViewBox {
                rect: tree_size.to_rect(0.0, 0.0),
                aspect: usvg::AspectRatio::default(),
            },
            root: usvg::Node::new(usvg::NodeKind::Group(usvg::Group::default())),
        };

        let oversize = piece_width.min(piece_height) as f64 * PIECE_OVERSIZE_RATIO;
        let mut n_oversize = 0.0;
        let mut s_oversize = 0.0;
        let mut e_oversize = 0.0;
        let mut w_oversize = 0.0;

        if row > 0 {
            n_oversize = oversize;
        }

        if row < puzzle.puzzle_height() - 1 {
            s_oversize = oversize;
        }

        if col > 0 {
            w_oversize = oversize;
        }

        if col < puzzle.puzzle_width() - 1 {
            e_oversize = oversize;
        }

        let mut path_data = usvg::PathData::new();
        let mut cursor_x = (west_tab * tab_width + border_size) as f64 - w_oversize;
        let mut cursor_y = (north_tab * tab_height + border_size) as f64 - n_oversize;

        // start in northwest corner
        path_data.push_move_to(cursor_x, cursor_y);

        let mut rel_line = |dx: f64, dy: f64| {
            cursor_x += dx;
            cursor_y += dy;
            path_data.push_line_to(cursor_x, cursor_y);
        };

        let piece_width: f64 = piece_width.into();
        let piece_height: f64 = piece_height.into();

        let tab_width: f64 = tab_width.into();
        let tab_height: f64 = tab_height.into();

        let mut ns_tab_inner_size: f64 = (TAB_INNER_SIZE_RATIO * piece_width).round();
        if ns_tab_inner_size / 2.0 != 0.0 {
            ns_tab_inner_size -= 1.0;
        }

        let mut ns_tab_outer_size: f64 = (TAB_OUTER_SIZE_RATIO * piece_width).round();
        if ns_tab_outer_size / 2.0 != 0.0 {
            ns_tab_outer_size -= 1.0;
        }

        let ns_corner_seg_size = (piece_width - ns_tab_inner_size) / 2.0;
        let ns_bulge_half_size = (ns_tab_outer_size - ns_tab_inner_size) / 2.0;

        let mut ew_tab_inner_size: f64 = (TAB_INNER_SIZE_RATIO * piece_height).round();
        if ew_tab_inner_size / 2.0 != 0.0 {
            ew_tab_inner_size -= 1.0;
        }

        let mut ew_tab_outer_size: f64 = (TAB_OUTER_SIZE_RATIO * piece_height).round();
        if ew_tab_outer_size / 2.0 != 0.0 {
            ew_tab_outer_size -= 1.0;
        }

        let ew_corner_seg_size = (piece_height - ew_tab_inner_size) / 2.0;
        let ew_bulge_half_size = (ew_tab_outer_size - ew_tab_inner_size) / 2.0;

        // northern eastward path
        rel_line(w_oversize + ns_corner_seg_size - n_oversize, 0.0);

        if north_tab > 0 {
            rel_line(-ns_bulge_half_size, -tab_height);
            rel_line(ns_tab_outer_size + 2.0 * n_oversize, 0.0);
            rel_line(-ns_bulge_half_size, tab_height);
        } else if north_blank > 0 {
            rel_line(2.0 * n_oversize, 0.0);
            rel_line(-ns_bulge_half_size - n_oversize, tab_height + n_oversize);
            rel_line(ns_tab_outer_size, 0.0);
            rel_line(-ns_bulge_half_size - n_oversize, -tab_height - n_oversize);
            rel_line(2.0 * n_oversize, 0.0);
        } else {
            rel_line(ns_tab_inner_size, 0.0);
        }

        rel_line(ns_corner_seg_size - n_oversize + e_oversize, 0.0);

        // eastern southward path
        rel_line(0.0, n_oversize + ew_corner_seg_size - e_oversize);

        if east_tab > 0 {
            rel_line(tab_width, -ew_bulge_half_size);
            rel_line(0.0, ew_tab_outer_size + 2.0 * e_oversize);
            rel_line(-tab_width, -ew_bulge_half_size);
        } else if east_blank > 0 {
            rel_line(0.0, 2.0 * e_oversize);
            rel_line(-tab_width - e_oversize, -ew_bulge_half_size - e_oversize);
            rel_line(0.0, ew_tab_outer_size);
            rel_line(tab_width + e_oversize, -ew_bulge_half_size - e_oversize);
            rel_line(0.0, 2.0 * e_oversize);
        } else {
            rel_line(0.0, ew_tab_inner_size);
        }

        rel_line(0.0, ew_corner_seg_size - e_oversize + s_oversize);

        // southern westward path
        rel_line(-e_oversize - ns_corner_seg_size + s_oversize, 0.0);

        if south_tab > 0 {
            rel_line(ns_bulge_half_size, tab_height);
            rel_line(-ns_tab_outer_size - 2.0 * s_oversize, 0.0);
            rel_line(ns_bulge_half_size, -tab_height);
        } else if south_blank > 0 {
            rel_line(-2.0 * s_oversize, 0.0);
            rel_line(ns_bulge_half_size + s_oversize, -tab_height - s_oversize);
            rel_line(-ns_tab_outer_size, 0.0);
            rel_line(ns_bulge_half_size + s_oversize, tab_height + s_oversize);
            rel_line(-2.0 * s_oversize, 0.0);
        } else {
            rel_line(-ns_tab_inner_size, 0.0);
        }

        rel_line(s_oversize - ns_corner_seg_size - w_oversize, 0.0);

        // western northward path
        rel_line(0.0, -s_oversize - ew_corner_seg_size + w_oversize);

        if west_tab > 0 {
            rel_line(-tab_width, ew_bulge_half_size);
            rel_line(0.0, -ew_tab_outer_size - 2.0 * w_oversize);
            rel_line(tab_width, ew_bulge_half_size);
        } else if west_blank > 0 {
            rel_line(0.0, -2.0 * w_oversize);
            rel_line(tab_width + w_oversize, ew_bulge_half_size + w_oversize);
            rel_line(0.0, -ew_tab_outer_size);
            rel_line(-tab_width - w_oversize, ew_bulge_half_size + w_oversize);
            rel_line(0.0, -2.0 * w_oversize);
        } else {
            rel_line(0.0, -ew_tab_inner_size);
        }

        rel_line(0.0, w_oversize - ew_corner_seg_size - n_oversize);

        tree.root.append_kind(usvg::NodeKind::Path(usvg::Path {
            fill: Some(usvg::Fill::default()), // black
            data: Rc::new(path_data),
            //rendering_mode: usvg::ShapeRendering::CrispEdges,
            ..usvg::Path::default()
        }));

        let mut mask = resvg::tiny_skia::Pixmap::new(sprite_width, sprite_height).unwrap();
        resvg::render(
            &tree,
            usvg::FitTo::Original,
            tiny_skia::Transform::default(),
            mask.as_mut(),
        );

        for (x, y, pixel) in crop.enumerate_pixels_mut() {
            pixel.channels_mut()[3] = mask.pixel(x, y).unwrap().alpha();
        }

        (crop, sprite_origin_x, sprite_origin_y)
    }

    pub fn index(&self) -> PieceIndex {
        self.index
    }

    pub fn kind(&self) -> PieceKind {
        self.kind
    }

    pub fn sprite_clone(&self) -> crate::image::Image {
        self.sprite.clone()
    }

    pub fn sprite_origin_x(&self) -> f64 {
        self.sprite_origin_x
    }

    pub fn sprite_origin_y(&self) -> f64 {
        self.sprite_origin_y
    }

    pub fn transform(&self) -> bevy::prelude::Transform {
        self.transform
    }
}
