use ::rand::{
    seq::SliceRandom,
    prelude::IteratorRandom
};
use macroquad::prelude::*;


const CELL_SIZE: f32 = 50.;
const BIG_FONT_SIZE: f32 = 40.;
const SMALL_FONT_SIZE: f32 = 20.;
const TEXT_FONT_SIZE: f32 = 20.;

const BIG_NUM_OFFSET: (f32, f32) = (10., 40.);
const SMALL_NUM_OFFSET: (f32, f32) = (5., 18.);

const SMALL_LINES_THICKNESS: f32 = 1.;
const BIG_LINES_THICKNESS: f32 = 3.;

const BACKGROUND_COLOR: Color = BLACK;
const GRID_COLOR: Color = WHITE;
const BIG_NUM_COLOR: Color = WHITE;
const SMALL_NUM_COLOR: Color = WHITE;
const TEXT_COLOR: Color = WHITE;

const TICK_SECONDS: f64 = 0.2;

const RESET_GRID_KEY: KeyCode = KeyCode::Space;


#[derive(Debug, Clone)]
struct Cell {
    possible_values: Vec<u8>,
    propagated: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self { possible_values: (1..=9).collect(), propagated: false }
    }
}

impl Cell {
    fn collapse(&mut self) -> u8 {
        if self.possible_values.len() > 1 {
            let value = *self.possible_values.choose(&mut ::rand::thread_rng()).unwrap();
            self.possible_values = vec![value];
        }
        self.possible_values[0]
    }

    fn remove_possibility(&mut self, value: u8) -> Result<(), ()> {
        if self.possible_values.len() > 1 {
            self.possible_values = self.possible_values
                .iter()
                .filter_map(|val|
                    if *val != value {
                        Some(*val)
                    } else {
                        None
                    }
                )
                .collect();
        } else {
            if self.possible_values[0] == value {
                return Err(());
            }
        }
        Ok(())
    }
}

struct Grid {
    cells: Vec<Cell>,
}

impl Grid {
    fn new() -> Self {
        Self {
            cells: vec![Cell::default(); 81]
        }
    }

    fn is_resolve(&self) -> bool {
        !self.cells.iter().any(|c| c.possible_values.len() > 1)
    }

    fn get_lowest_entropy_cell_idx(&self) -> usize {
        let min = self.cells
            .iter()
            .fold(9usize, |min, c| {
                if c.possible_values.len() == 1 {
                    min
                } else {
                    c.possible_values.len().min(min)
                }
            });

        self.cells
            .iter()
            .enumerate()
            .filter_map(|(i, c)|
                if c.possible_values.len() == min {
                    Some(i)
                } else {
                    None
                })
                .choose(&mut ::rand::thread_rng())
                .unwrap()
    }

    fn propagate(&mut self, idx: usize) -> Result<(), ()> {
        if self.cells[idx].possible_values.len() == 1 {
            self.cells[idx].propagated = true;
            let cell_value = self.cells[idx].possible_values[0];

            for idx in Grid::iter_col(idx) {
                if !self.cells[idx].propagated {
                    self.cells[idx].remove_possibility(cell_value)?;
                    self.propagate(idx)?;
                }
            }
            for idx in Grid::iter_row(idx) {
                if !self.cells[idx].propagated {
                    self.cells[idx].remove_possibility(cell_value)?;
                    self.propagate(idx)?;
                }
            }
            for idx in Grid::iter_square(idx) {
                if !self.cells[idx].propagated {
                    self.cells[idx].remove_possibility(cell_value)?;
                    self.propagate(idx)?;
                }
            }
        }
        Ok(())
    }

    fn iter_row(idx: usize) -> impl Iterator<Item = usize> {
        (0..81).filter(move |i| idx / 9 == i / 9)
    }

    fn iter_col(idx: usize) -> impl Iterator<Item = usize> {
        (0..81).filter(move |i| idx % 9 == i % 9)
    }

    fn iter_square(idx: usize) ->  impl Iterator<Item = usize> {
        let get_square_idx = |idx: usize| {
            27 * ((idx / 9) / 3) + 3 * ((idx % 9) / 3)
        };
        let square_idx = get_square_idx(idx);
        (0..81).filter(move |&i| get_square_idx(i) == square_idx)
    }

    fn end_propagation(&mut self) {
        self.cells.iter_mut().for_each(|c| c.propagated = false);
    }

    fn draw(&self) {
        let grid_position = (
            screen_width() / 2. - CELL_SIZE * 4.5,
            screen_height() / 2. - CELL_SIZE * 4.5,
        );
        for i in 0..10 {
            let thickness = if i % 3 == 0 {
                BIG_LINES_THICKNESS
            } else {
                SMALL_LINES_THICKNESS
            };
            draw_line(
                grid_position.0,
                grid_position.1 + i as f32 * CELL_SIZE,
                grid_position.0 + 9. * CELL_SIZE,
                grid_position.1 + i as f32 * CELL_SIZE,
                thickness,
                GRID_COLOR);
            draw_line(
                grid_position.0 + i as f32 * CELL_SIZE,
                grid_position.1,
                grid_position.0 + i as f32 * CELL_SIZE,
                grid_position.1 + 9. * CELL_SIZE,
                thickness,
                GRID_COLOR
            );
        }

        for idx in 0..81 {
            let values = &self.cells[idx].possible_values;

            if values.len() == 1 {
                draw_text(
                    &values[0].to_string(),
                    grid_position.0 + (idx % 9) as f32 * CELL_SIZE + BIG_NUM_OFFSET.0,
                    grid_position.1 + (idx / 9) as f32 * CELL_SIZE + BIG_NUM_OFFSET.1,
                    BIG_FONT_SIZE,
                    BIG_NUM_COLOR);
            } else {
                for (i, v) in values.iter().enumerate() {
                    draw_text(
                        &v.to_string(),
                        grid_position.0 + (idx % 9) as f32 * CELL_SIZE + (i % 3) as f32 * CELL_SIZE / 3.5 + SMALL_NUM_OFFSET.0,
                        grid_position.1 + (idx / 9) as f32 * CELL_SIZE + (i / 3) as f32 * CELL_SIZE / 3.5 + SMALL_NUM_OFFSET.1,
                        SMALL_FONT_SIZE,
                        SMALL_NUM_COLOR);
                }
            }
        }
    }
}

#[macroquad::main("Wave Function Collapse Sudoku")]
async fn main() {

    let mut grid = Grid::new();

    let mut tick = get_time();
    loop {
        clear_background(BACKGROUND_COLOR);

        if get_time() - tick > TICK_SECONDS {
            tick = get_time();
            if !grid.is_resolve() {
                let cell_idx = grid.get_lowest_entropy_cell_idx();
                grid.cells[cell_idx].collapse();
                if grid.propagate(cell_idx).is_ok() {
                    grid.end_propagation();
                } else {
                    // Reset grid in case of unresolvable cell
                    grid = Grid::new();
                }
            }
        }

        if is_key_pressed(RESET_GRID_KEY) {
            grid = Grid::new();
        }

        grid.draw();
        draw_text(
            &format!("Press [{:?}] to reset",
            RESET_GRID_KEY),
            0., TEXT_FONT_SIZE,
            TEXT_FONT_SIZE,
            TEXT_COLOR
        );

        next_frame().await;
    }
}
