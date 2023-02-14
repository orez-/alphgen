use std::collections::HashSet;

pub struct Sprite {
    // XXX: this should be &[u8] but i dont want to live in lifetime hell right now
    data: Box<[u8]>,
    width: usize,
    height: usize,
}

impl Sprite {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn find_contours(&self) -> Vec<Vec<(usize, usize)>> {
        use Direction::*;

        // find all edges
        let mut edges = HashSet::new();
        for x in 0..self.width {
            for y in 0..self.height {
                if self.index((x, y)) {
                    for edge in Edge::surrounding(x, y) {
                        if !edges.remove(&edge.complement()) {
                            edges.insert(edge);
                        }
                    }
                }
            }
        }

        // separate into chain loops
        let mut seen = HashSet::new();
        edges.iter().filter_map(|&start| {
            // Identify edge chain
            if !seen.insert(start) { return None; }
            let mut edge = start;
            let mut out = Vec::new();
            while {
                let (x, y) = edge.dest();
                edge = [Left, Up, Right].into_iter().filter_map(|dir| {
                    let direction = edge.direction.turn(dir);
                    let candidate = Edge { x, y, direction };
                    edges.contains(&candidate).then(|| candidate)
                }).next().expect("edges generated from pixel grid should have a looping path");
                seen.insert(edge);
                out.push(edge);
                edge != start
            } {}

            // Align loop with change in direction.
            // This ensures we combine as many edges as possible, that we aren't
            // accidentally straddling a single edge across the Vec-loop boundary.
            let offset = out.iter()
                .position(|edge| start.direction != edge.direction)
                .expect("edges should change direction at some point in the loop");
            out.rotate_left(offset);

            // Dedup colinear edges and convert to list of points
            out.dedup_by_key(|edge| edge.direction);
            let out = out.into_iter().map(|edge| (edge.x, edge.y)).collect();
            Some(out)
        }).collect()
    }

    fn index(&self, (x, y): (usize, usize)) -> bool {
        if x >= self.width {
            panic!("x: {x} must be less than width {}", self.width);
        }
        if y >= self.height {
            panic!("y: {y} must be less than height {}", self.height);
        }
        let idx = y * self.width + x;
        let byte = idx / 8;
        let offset = idx % 8;
        let bit = 1 << offset;
        self.data[byte] & bit != 0
    }
}

// - origin at top left
// - square is on the right of the edge
//    ^->
//    |#|
//    <-v
// - edge coordinates are from top left of pixel
// eg, `Edge { x: 2, y: 3, direction: Direction::Right }`
// is the edge above the pixel at (2, 3), which is pointing right.
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq, Hash)]
struct Edge {
    x: usize,
    y: usize,
    direction: Direction,
}

impl Edge {
    fn dest(&self) -> (usize, usize) {
        match self.direction {
            Direction::Up => (self.x, self.y - 1),
            Direction::Right => (self.x + 1, self.y),
            Direction::Down => (self.x, self.y + 1),
            Direction::Left => (self.x - 1, self.y),
        }
    }

    /// Return an array of the Edges surrounding the pixel at (`x`, `y`).
    fn surrounding(x: usize, y: usize) -> [Edge; 4] {
        [
            Edge { x, y: y + 1, direction: Direction::Up },
            Edge { x: x, y, direction: Direction::Right },
            Edge { x: x + 1, y, direction: Direction::Down },
            Edge { x: x + 1, y: y + 1, direction: Direction::Left },
        ]
    }

    fn complement(&self) -> Edge {
        let (x, y) = self.dest();
        let direction = self.direction.turn(Direction::Down);
        Edge { x, y, direction }
    }
}

#[derive(Clone, Copy)]
#[derive(PartialEq, Eq, Hash)]
enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl Direction {
    fn turn(self, by: Direction) -> Direction {
        let dir = self as u8;
        let by = by as u8;
        let dir = (dir + by) % 4;
        dir.into()
    }
}

impl From<u8> for Direction {
    fn from(num: u8) -> Self {
        match num {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            _ => panic!("invalid value for Direction: {num}"),
        }
    }
}

// ===

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    fn normalize_contour(contour: &mut [(usize, usize)]) {
        let mut curr: VecDeque<(usize, usize)> = contour.iter().copied().collect();
        let mut best = curr.clone();
        let mut rotation = 0;
        for r in 1..contour.len() {
            curr.rotate_left(1);
            if curr < best {
                best = curr.clone();
                rotation = r;
            }
        }
        contour.rotate_left(rotation);
    }

    fn normalize_contours(contours: &mut [Vec<(usize, usize)>]) {
        for c in contours.iter_mut() {
            normalize_contour(c);
        }
        contours.sort_unstable();
    }

    #[test]
    fn serializes_a() {
        let a = Box::new([
            0b00010000,
            0b00101000,
            0b00101000,
            0b01000100,
            0b01111100,
            0b10000010,
            0b10000010,
            0b00000000,
        ]);
        let sprite = Sprite {
            data: a,
            width: 8,
            height: 8,
        };
        let mut actual = sprite.find_contours();
        let mut expected = vec![
            vec![
                (2, 3), (3, 3), (3, 1), (4, 1), (4, 0), (5, 0), (5, 1), (6, 1), (6, 3), (7, 3),
                (7, 5), (8, 5), (8, 7), (7, 7), (7, 5), (2, 5), (2, 7), (1, 7), (1, 5), (2, 5)
            ],
            vec![(4, 3), (3, 3), (3, 4), (6, 4), (6, 3), (5, 3), (5, 1), (4, 1)],
        ];

        normalize_contours(&mut actual);
        normalize_contours(&mut expected);
        assert_eq!(actual, expected);
    }
}
