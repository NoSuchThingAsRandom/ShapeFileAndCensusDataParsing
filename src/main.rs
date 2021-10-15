extern crate num_traits;
// Source: https://webarchive.nationalarchives.gov.uk/ukgwa/20160107193025/http://www.ons.gov.uk/ons/guide-method/geography/beginner-s-guide/census/output-area--oas-/index.html
// We ignore the inner rings of polygons?
extern crate polylabel;


use std::time::Instant;

use geo_types::{Coordinate, LineString};
use plotters::coord::Shift;
use plotters::prelude::*;
use polylabel::polylabel;
use shapefile::Shape;
use shapefile::dbase::{FieldValue, Record};

const DEBUG_ITERATION: usize = 500;
const GRID_SIZE: u32 = 16384;
const X_OFFSET: i32 = 75000;
const Y_OFFSET: i32 = 1000;

fn convert_geo_point_to_pixel(coord: Coordinate<f64>) -> (i32, i32) {
    (
        (coord.x - X_OFFSET as f64) as i32 / 45,
        GRID_SIZE as i32 - (coord.y - Y_OFFSET as f64) as i32 / 45,
    )
}

#[derive(Debug)]
struct Area {
    centre: Option<Coordinate<f64>>,
    points: geo_types::Polygon<f64>,
    label: String,
    code: String,
    name: String,
    alt_name: String,
}

impl Area {
    fn new_from_record(record: Record, polygon: geo_types::Polygon<f64>) -> Area {
        let label_record = record.get("label").expect("Missing required field 'label'");
        let label;
        if let FieldValue::Character(option_val) = label_record {
            label = option_val.clone().unwrap_or_else(|| String::from(""));
        } else {
            panic!("Unexpected field value type for label: {}", label_record);
        }


        let code_record = record.get("code").expect("Missing required field 'code'");
        let code;
        if let FieldValue::Character(option_val) = code_record {
            code = option_val.clone().unwrap_or_else(|| String::from(""));
        } else {
            panic!("Unexpected field value type for code: {}", code_record);
        }


        let name_record = record.get("name").expect("Missing required field 'name'");
        let name;
        if let FieldValue::Character(option_val) = name_record {
            name = option_val.clone().unwrap_or_else(|| String::from(""));
        } else {
            panic!("Unexpected field value type for name: {}", name_record);
        }

        let alt_name_record = record.get("altname").expect("Missing required field 'alt_name'");
        let alt_name;
        if let FieldValue::Character(option_val) = alt_name_record {
            alt_name = option_val.clone().unwrap_or_else(|| String::from(""));
        } else {
            panic!("Unexpected field value type for alt_name: {}", alt_name_record);
        }

        Area {
            centre: None,
            points: polygon,
            code,
            label,
            name,
            alt_name,
        }
    }
    /// Retrieves the center point or calculates the centre
    /// Also stores the result
    fn find_centre_point(&mut self) -> Coordinate<f64> {
        if self.centre.is_none() {
            self.centre = Some(Coordinate::from(polylabel(&self.points, &0.1).unwrap()));
        }
        self.centre.unwrap()
    }
    /// Retrieves the center point or calculates the centre
    /// DOES NOT CACHE THE RESULT
    fn get_centre_point(&self) -> Coordinate<f64> {
        self.centre.unwrap_or_else(|| Coordinate::from(polylabel(&self.points, &0.1).unwrap()))
    }
}


struct Map {
    data: Vec<Area>,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

impl Map {
    fn default() -> Map {
        Map { data: Vec::default(), min_x: i32::MAX, min_y: i32::MAX, max_x: i32::MIN, max_y: i32::MIN }
    }
    fn from_file(filename: &str) -> Map {
        //census_map_areas/England_wa_2011/england_wa_2011.shp
        let mut map = Map::default();
        let mut reader =
            shapefile::Reader::from_path(filename)
                .unwrap();
        let start_time = Instant::now();
        for (index, shape_record) in reader.iter_shapes_and_records().enumerate() {
            let (shape, record) = shape_record.unwrap();
            if let Shape::Polygon(polygon) = shape {
                assert!(!polygon.rings().is_empty());
                let rings: Vec<Coordinate<f64>>;
                let mut interior_ring;
                if polygon.rings().len() == 1 {
                    rings = polygon.rings()[0].points().iter().map(|p| geo_types::Coordinate::from(*p)).collect();
                    interior_ring = Vec::new();
                } else {
                    interior_ring = polygon.rings().iter().map(|r| LineString::from(r.points().iter().map(|p| geo_types::Coordinate::from(*p)).collect::<Vec<Coordinate<f64>>>())).collect();
                    rings = interior_ring.pop().unwrap().0;
                }
                let new_poly = geo_types::Polygon::new(LineString::from(rings), interior_ring);
                map.data.push(Area::new_from_record(record, new_poly));
            } else {
                panic!("Unexpected shape: {}", shape);
            }
            if index % DEBUG_ITERATION == 0 {
                println!("At index {} with time {:?}", index, start_time.elapsed());
            }
        }
        map
    }

    fn draw_with_labels<T: plotters::prelude::DrawingBackend>(&self, drawing_area: DrawingArea<T, Shift>) {
        self.draw(drawing_area, true);
    }
    fn draw<T: plotters::prelude::DrawingBackend>(&self, drawing_area: DrawingArea<T, Shift>, show_labels: bool) {
        let style = TextStyle::from(("sans-serif", 20).into_font()).color(&RED);
        for data in &self.data {
            if show_labels {
                let centre = data.centre.unwrap_or_else(|| data.get_centre_point());
                let centre = convert_geo_point_to_pixel(centre);
                drawing_area.draw_text(&data.label, &style, centre).unwrap();
            }
            let polygon = &data.points;
            for coords in &polygon.exterior().0 {
                let coords = convert_geo_point_to_pixel(*coords);
                //println!("Drawing pixel at: {} {:?}", point, coords);
                if coords.0 > GRID_SIZE as i32 {
                    panic!("X coord is too big! Coord: {}", coords.0);
                } else if coords.1 > GRID_SIZE as i32 {
                    panic!("Y coord is too big! Coord: {}", coords.1);
                } else if coords.0 < 0 {
                    panic!("X coord is too small! Coord: {}", coords.0);
                } else if coords.1 < 0 {
                    panic!("Y coord is too small! Coord: {}", coords.1);
                } else {
                    //println!("Drawing pixel at {:?}", coords);
                    drawing_area.draw_pixel(coords, &BLACK).unwrap();
                }
            }
        }
        drawing_area.present().unwrap();
    }
}

fn main() {
    println!("Hello, world!");

    let map = Map::from_file("census_map_areas/England_wa_2011/england_wa_2011.shp");
    let draw_backend =
        BitMapBackend::new("grid.png", (GRID_SIZE, GRID_SIZE)).into_drawing_area();
    draw_backend.fill(&WHITE).unwrap();
    map.draw(draw_backend, false);
    //load_data();
    print!("Done");
}
