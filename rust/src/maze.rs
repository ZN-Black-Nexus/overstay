// Directions, indexed consistently across the module: 0=North(-y), 1=East(+x), 2=South(+y), 3=West(-x).
const DX: [i32; 4] = [0, 1, 0, -1];
const DY: [i32; 4] = [-1, 0, 1, 0];
const OPPOSITE: [usize; 4] = [2, 3, 0, 1];

#[derive(Clone, Copy, Default)]
pub struct Cell {
    /// Whether a passage exists in each of the 4 directions (no wall there).
    pub open: [bool; 4],
}

pub struct Maze {
    pub w: i32,
    pub h: i32,
    pub cells: Vec<Cell>,
}

impl Maze {
    pub fn index(&self, x: i32, y: i32) -> usize {
        (y * self.w + x) as usize
    }

    pub fn cell(&self, x: i32, y: i32) -> Cell {
        self.cells[self.index(x, y)]
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h
    }

    /// Randomized depth-first search ("recursive backtracker") — produces a perfect
    /// maze (every cell reachable, no loops), which fits the disorienting,
    /// easy-to-lose-track-of-yourself Backrooms feel.
    pub fn generate(w: i32, h: i32) -> Self {
        let mut cells = vec![Cell::default(); (w * h) as usize];
        let mut visited = vec![false; (w * h) as usize];
        let mut stack: Vec<(i32, i32)> = Vec::new();

        let start = (0, 0);
        stack.push(start);
        visited[(start.1 * w + start.0) as usize] = true;

        while let Some(&(x, y)) = stack.last() {
            let mut order = [0usize, 1, 2, 3];
            // Fisher-Yates shuffle over the 4 directions.
            for i in (1..order.len()).rev() {
                let j = rand::random_range(0..=i);
                order.swap(i, j);
            }

            let mut advanced = false;
            for &dir in order.iter() {
                let (nx, ny) = (x + DX[dir], y + DY[dir]);
                if Self::in_bounds_static(nx, ny, w, h) {
                    let nidx = (ny * w + nx) as usize;
                    if !visited[nidx] {
                        let cidx = (y * w + x) as usize;
                        cells[cidx].open[dir] = true;
                        cells[nidx].open[OPPOSITE[dir]] = true;
                        visited[nidx] = true;
                        stack.push((nx, ny));
                        advanced = true;
                        break;
                    }
                }
            }

            if !advanced {
                stack.pop();
            }
        }

        Maze { w, h, cells }
    }

    fn in_bounds_static(x: i32, y: i32, w: i32, h: i32) -> bool {
        x >= 0 && y >= 0 && x < w && y < h
    }

    /// Breadth-first search from `start`, returning the farthest cell and its distance.
    /// Used to place the extraction door as far from spawn as the maze allows.
    pub fn farthest_from(&self, start: (i32, i32)) -> (i32, i32) {
        let mut dist = vec![-1i32; self.cells.len()];
        let mut queue = std::collections::VecDeque::new();

        let start_idx = self.index(start.0, start.1);
        dist[start_idx] = 0;
        queue.push_back(start);

        let mut farthest = start;
        let mut farthest_dist = 0;

        while let Some((x, y)) = queue.pop_front() {
            let d = dist[self.index(x, y)];
            if d > farthest_dist {
                farthest_dist = d;
                farthest = (x, y);
            }

            let cell = self.cell(x, y);
            for dir in 0..4 {
                if cell.open[dir] {
                    let (nx, ny) = (x + DX[dir], y + DY[dir]);
                    if self.in_bounds(nx, ny) {
                        let nidx = self.index(nx, ny);
                        if dist[nidx] == -1 {
                            dist[nidx] = d + 1;
                            queue.push_back((nx, ny));
                        }
                    }
                }
            }
        }

        farthest
    }
}
