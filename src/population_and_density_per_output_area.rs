use std::convert::TryFrom;
use std::iter::Map;

use enum_map::EnumMap;
use log::debug;
use serde::{Deserialize, Deserializer};
use serde::de::Error;

use crate::parsing_error::{ParsingError, ParsingErrorType};

pub const SELECTED_COLUMNS: &str = "GEOGRAPHY_NAME,GEOGRAPHY_TYPE,RURAL_URBAN_NAME,RURAL_URBAN_TYPECODE,CELL_NAME,MEASURES_NAME,OBS_VALUE,OBS_STATUS,RECORD_OFFSET,RECORD_COUNT";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct PreProcessingRecord {
    pub geography_name: String,
    geography_type: String,
    rural_urban_name: String,
    cell_name: String,
    measures_name: String,
    obs_value: String,
    obs_status: String,
    record_offset: u32,
    record_count: u32,
}

#[derive(Deserialize, Debug, Enum)]
pub enum AreaClassification {
    #[serde(alias = "Total")]
    Total,
    #[serde(alias = "Urban (total)")]
    UrbanTotal,
    #[serde(alias = "Urban major conurbation")]
    UrbanMajorConurbation,
    #[serde(alias = "Urban minor conurbation")]
    UrbanMinorConurbation,
    #[serde(alias = "Urban city and town")]
    UrbanCity,
    #[serde(alias = "Urban city and town in a sparse setting")]
    UrbanSparseTownCity,
    #[serde(alias = "Rural (total)")]
    RuralTotal,
    #[serde(alias = "Rural town and fringe")]
    RuralTown,
    #[serde(alias = "Rural town and fringe in a sparse setting")]
    RuralSparseTown,
    #[serde(alias = "Rural village")]
    RuralVillage,
    #[serde(alias = "Rural village in a sparse setting")]
    RuralSparseVillage,
    #[serde(alias = "Rural hamlet and isolated dwellings")]
    RuralHamlet,
    #[serde(alias = "Rural hamlet and isolated dwellings in a sparse setting")]
    RuralSparseHamlet,
}

#[derive(Deserialize, Debug, Enum)]
pub enum PersonType {
    #[serde(alias = "All usual residents")]
    All,
    #[serde(alias = "Males")]
    Male,
    #[serde(alias = "Females")]
    Female,
    #[serde(alias = "Lives in a household")]
    LivesInHousehold,
    #[serde(alias = "Lives in a communal establishment")]
    LivesInCommunalEstablishment,
    #[serde(alias = "Schoolchild or full-time student aged 4 and over at their non term-time address")]
    Schoolchild,
}

#[derive(Debug)]
pub struct PopulationRecord {
    pub geography_code: String,
    pub geography_type: String,
    pub area_size: f32,
    pub density: f32,
    pub population_counts: EnumMap<AreaClassification, EnumMap<PersonType, u16>>,
}


impl TryFrom<Vec<PreProcessingRecord>> for PopulationRecord {
    type Error = ParsingError;

    fn try_from(records: Vec<PreProcessingRecord>) -> Result<Self, Self::Error> {
        if records.is_empty() {
            return Err(ParsingError::new(ParsingErrorType::InvalidDataType(String::from("Array is empty")), Some(String::from("Need at least one record, to build a Population Record"))));
        }
        let geography_code = String::from(&records[0].geography_name);
        let geography_type = String::from(&records[0].geography_type);
        let mut area_size: f32 = 0.0;
        let mut density: f32 = 0.0;
        let mut data: EnumMap<AreaClassification, EnumMap<PersonType, u16>> = EnumMap::default();
        for record in records {
            if record.geography_name != geography_code {
                return Err(ParsingError::new(ParsingErrorType::InvalidDataType(String::from(&record.geography_name)), Some(format!("Mis matching geography codes: {} and {}", geography_code, record.geography_name))));
            }
            if record.geography_type != geography_type {
                return Err(ParsingError::new(ParsingErrorType::InvalidDataType(String::from(&record.geography_type)), Some(format!("Mis matching geography type: {} and {}", geography_type, record.geography_type))));
            }
            if record.measures_name == "Value" {
                if &record.cell_name == "Area (Hectares)" {
                    area_size = record.obs_value.parse().unwrap_or(0.0);
                } else if &record.cell_name == "Density (number of persons per hectare)" {
                    density = record.obs_value.parse().unwrap_or(0.0);
                } else {
                    let area_classification: AreaClassification = serde_plain::from_str(&record.rural_urban_name)?;
                    let person_classification: PersonType = serde_plain::from_str(&record.cell_name)?;
                    data[area_classification][person_classification] = record.obs_value.parse()?;
                }
            }
        }
        debug!("New record: Code: {}, data: {:?}", geography_code, &data[AreaClassification::Total]);

        Ok(PopulationRecord {
            geography_code,
            geography_type,
            area_size,
            density,
            population_counts: data,
        })
    }
}