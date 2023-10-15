mod utils;
use image::{ImageBuffer, Rgba};
use js_sys;
use std::fmt;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cell::Dead => write!(f, "Dead"),
            Cell::Alive => write!(f, "Alive"),
        }
    }
}

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen(raw_module = "../www/index.js")]
extern "C" {
    fn paint(imageData: Vec<u8>);
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32) -> Universe {
        utils::set_panic_hook();

        let cells: Vec<Cell> = vec![Cell::Dead; (width * height) as usize]
            .iter()
            .map(|_| {
                if js_sys::Math::random() < 0.2 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
            image_buffer: ImageBuffer::new(width * 4, height * 4),
        }
    }

    fn cells(&self) -> Vec<Cell> {
        self.cells.clone()
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        let overflowed_row = row % self.height;
        let overflowed_column = column % self.width;
        (overflowed_row * self.width + overflowed_column) as usize
    }

    fn get_pos(&self, index: u32) -> (usize, usize) {
        let row = index / self.width;
        let column = index % self.width;
        (row as usize, column as usize)
    }

    fn live_neighbor_count(&self, row: u32, col: u32) -> u8 {
        vec![
            (row - 1, col - 1),
            (row - 1, col),
            (row - 1, col + 1), // top neighbors
            (row, col - 1),
            (row, col + 1), // left and right neighbors
            (row + 1, col - 1),
            (row + 1, col),
            (row + 1, col + 1), // bottom neighbors
        ]
        .iter()
        .filter(|&&(r, c)| r < self.height && c < self.width)
        .map(|(r, c)| {
            let index = self.get_index(*r, *c);
            self.cells[index] as u8
        })
        .sum()
    }

    pub fn tick(&mut self) {
        self.cells = self
            .cells
            .clone()
            .iter()
            .enumerate()
            .map(|(index, cell)| {
                let (row, col) = self.get_pos(index as u32);
                let live_neighbors = self.live_neighbor_count(row as u32, col as u32);
                match (cell, live_neighbors) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (_, _) => *cell,
                }
            })
            .collect();
    }

    pub fn paint_state(&mut self) {
        self.cells().iter().enumerate().for_each(|(index, cell)| {
            let color = match cell {
                Cell::Alive => vec![
                    Rgba([141, 173, 130, 255]),
                    Rgba([181, 110, 48, 255]),
                    Rgba([135, 173, 163, 255]),
                ][(js_sys::Math::random() * 3.0) as usize],
                Cell::Dead => Rgba([40, 40, 40, 255]),
            };

            let (row, col) = self.get_pos(index as u32);

            let pixel_x = col * 4;
            let pixel_y = row * 4;

            (0..4).for_each(|dx| {
                (0..4).for_each(|dy| {
                    self.image_buffer.put_pixel(
                        (pixel_x + dx) as u32,
                        (pixel_y + dy) as u32,
                        color,
                    );
                });
            });
        });

        paint(self.image_buffer.clone().into_raw());
    }
}
