use std::io::{BufReader, Write, Read};
use std::fs::{self, DirBuilder, File};

use uuid::Uuid;
use bincode::{serialize, deserialize, Infinite};
use serde::{Serialize, Deserialize};

const REGION_DEPTH: u32 = 16;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Map {
    id: Uuid,
    width: u32,
    height: u32,
    region_size: u32,
    regions: Vec<Vec<Region>>,
    seed: i32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum BiomeType {
    Arid,
    Grassland,
    Ocean,
    Rocky,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Region {
    id: Uuid,
    width: u32,
    height: u32,
    biome: BiomeType,
    tiles: Vec<Vec<Vec<Tile>>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum TileType {
    Air,
    Grass,
    Sand,
    Soil,
    Stone,
    Water,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Tile {
    pub solid: bool,
    pub tile_type: TileType,
}

impl Map {
    pub fn new(width: u32, height: u32, region_size: u32) -> Map {
        Map {
            id: Uuid::new_v4(),
            width: width,
            height: height,
            region_size: region_size,
            regions: Vec::<Vec<Region>>::new(),
            seed: -1,
        }
    }

    pub fn set_seed(&mut self, seed: i32) {
        self.seed = seed;
    }

    pub fn generate_regions<F>(&mut self, progress_callback: F)
    where
        F: Fn(i32),
    {
        let region_count = (self.width * self.height) as f32;
        let mut current_index = 0;

        for _ in 0..self.width {
            let mut column = Vec::<Region>::new();
            for _ in 0..self.height {
                column.push(Region::new(self.region_size, BiomeType::Arid));

                current_index += 1;
                let progress = ((current_index as f32 / region_count) as i32) * 100;
                progress_callback(progress);
            }
            self.regions.push(column);
        }
    }

    pub fn save(&mut self) {
        let path = format!("maps/{}/", self.id);
        let mut dir = DirBuilder::new()
            .recursive(true)
            .create(path.clone())
            .unwrap();

        for mut c in self.regions.iter_mut() {
            for mut r in c.iter_mut() {
                r.save_tiles(path.clone());
                r.dispose_tiles();
            }
        }

        let fname = format!("{}{}.map", path, self.id);
        let mut file = File::create(fname).unwrap();
        let encoded = serialize(&self, Infinite).unwrap();
        file.write_all(encoded.as_slice());
    }

    pub fn load(&mut self, id: String) {
        let path = format!("maps/{}/", id);
        let file = File::open(format!("{}{}.map", path.clone(), id)).unwrap();
        let mut buffer = Vec::<u8>::new();
        let encoded = BufReader::new(file).read_to_end(&mut buffer).unwrap();
        *self = deserialize(&buffer).unwrap();

        for mut c in self.regions.iter_mut() {
            for mut r in c.iter_mut() {
                r.load_tiles(path.clone());
            }
        }
    }
}

impl Region {
    pub fn new(size: u32, biome: BiomeType) -> Region {
        Region {
            id: Uuid::new_v4(),
            width: size,
            height: size,
            biome: biome,
            tiles: Vec::<Vec<Vec<Tile>>>::new(),
        }
    }

    pub fn generate_tiles(&mut self) {
        for _ in 0..self.width {
            let mut column = Vec::<Vec<Tile>>::new();
            for _ in 0..self.height {
                let mut width = Vec::<Tile>::new();

                for z in 0..REGION_DEPTH {
                    // TODO: Replace this with map generation logic
                    if z > REGION_DEPTH / 2 {
                        width.push(Tile {
                            solid: false,
                            tile_type: TileType::Air,
                        });
                    } else {
                        width.push(Tile {
                            solid: true,
                            tile_type: TileType::Stone,
                        });
                    }
                }

                column.push(width);
            }
            self.tiles.push(column);
        }
    }

    pub fn load_tiles(&mut self, dir: String) {
        let file = File::open(format!("{}{}.region", dir, self.id)).unwrap();
        let mut buffer = Vec::<u8>::new();
        let encoded = BufReader::new(file).read_to_end(&mut buffer).unwrap();
        self.tiles = deserialize(&buffer).unwrap();
    }

    pub fn save_tiles(&self, dir: String) {
        let fname = format!("{}{}.region", dir, self.id);
        let mut file = File::create(fname).unwrap();
        let encoded = serialize(&self.tiles, Infinite).unwrap();
        file.write_all(encoded.as_slice());
    }

    pub fn dispose_tiles(&mut self) {
        self.tiles.clear();
    }
}
