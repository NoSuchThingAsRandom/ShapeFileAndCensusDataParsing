use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::iter::Map;
use std::time::Instant;
use log::{debug, error, info, warn};
use serde_json::{Number, Value};

use crate::parsing_error::{ParsingError, ParsingErrorType};
use crate::population_and_density_per_output_area::{PopulationRecord, PreProcessingRecord, SELECTED_COLUMNS};

const ENGLAND_OUTPUT_AREAS_CODE: &str = "2092957699TYPE299";
const NOMIS_API: &str = "https://www.nomisweb.co.uk/api/v01/";

pub struct TableInfo {
    id: String,
    coded_name: String,
    source: String,
    metadata: String,
    keywords: Vec<String>,
    geo_level: Vec<String>,
}


impl TryFrom<HashMap<String, String>> for TableInfo {
    type Error = ParsingError;

    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let id = value.get("id").ok_or_else(|| ParsingError::new(ParsingErrorType::MissingKey, Some("content/type".to_string())))?.to_string();
        let source = value.get("contenttype/sources").ok_or_else(|| ParsingError::new(ParsingErrorType::MissingKey, Some("content/type".to_string())))?.to_string();
        let coded_name = value.get("Mnemonic").ok_or_else(|| ParsingError::new(ParsingErrorType::MissingKey, Some("content/type".to_string())))?.to_string();
        let metadata = value.get("MetadataText0").ok_or_else(|| ParsingError::new(ParsingErrorType::MissingKey, Some("content/type".to_string())))?.to_string();
        let keywords = value.get("Keywords").ok_or_else(|| ParsingError::new(ParsingErrorType::MissingKey, Some("content/type".to_string())))?.split(",").map(|s| s.to_string()).collect();
        let geo_level = value.get("contenttype/geoglevel").ok_or_else(|| ParsingError::new(ParsingErrorType::MissingKey, Some("content/type".to_string())))?.split(",").map(|s| s.to_string()).collect();
        Ok(TableInfo {
            id,
            coded_name,
            source,
            metadata,
            keywords,
            geo_level,
        })
    }
}

impl Display for TableInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID: {}, Coded Name: {}, Source: {}, Keywords: ({:?}), Geo Levels: {:?}", self.id, self.coded_name, self.source, self.keywords, self.geo_level)
    }
}

impl Debug for TableInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub struct DataFetcher {
    client: reqwest::Client,
}

impl Default for DataFetcher {
    fn default() -> Self {
        DataFetcher { client: reqwest::Client::default() }
    }
}

fn extract_value_from_json<'a>(object: &'a Value, name: &str) -> Result<&'a Value, ParsingError> {
    let object = object.get(name).ok_or_else(|| ParsingError::new(ParsingErrorType::MissingKey, Some(name.to_string())))?;
    Ok(object)
}

fn extract_string_from_json(object: &Value, name: &str) -> Result<String, ParsingError> {
    let object = object.get(name).ok_or_else(|| ParsingError::new(ParsingErrorType::MissingKey, Some(name.to_string())))?;
    if let Value::Number(n) = object {
        return Ok(n.to_string());
    }
    let object = object.as_str().ok_or_else(|| ParsingError::new(ParsingErrorType::InvalidDataType(String::from("String")), Some(name.to_string())))?;
    Ok(object.to_string())
}

fn extract_array_from_json<'a>(object: &'a Value, name: &str) -> Result<&'a Vec<Value>, ParsingError> {
    let object = object.get(name).ok_or_else(|| ParsingError::new(ParsingErrorType::MissingKey, Some(name.to_string())))?;
    let object = object.as_array().ok_or_else(|| ParsingError::new(ParsingErrorType::InvalidDataType(String::from("Array")), Some(name.to_string())))?;
    Ok(object)
}

fn extract_map_from_json<'a>(object: &'a Value, name: &str) -> Result<&'a serde_json::Map<String, Value>, ParsingError> {
    let object = object.get(name).ok_or_else(|| ParsingError::new(ParsingErrorType::MissingKey, Some(name.to_string())))?;
    let object = object.as_object().ok_or_else(|| ParsingError::new(ParsingErrorType::InvalidDataType(String::from("Array: ")), Some(name.to_string())))?;
    Ok(object)
}


impl DataFetcher {
    pub async fn get_datasets(&self) -> Result<Value, ParsingError> {
        let api: String = format!("{}dataset/def.sdmx.json?search=c2011*", NOMIS_API);
        println!("Making request to: {}", api);
        let request = self.client.get(api).send().await?;
        println!("Got response: {:?}", request);
        let data = request.text().await?;
        let json: Value = serde_json::from_str(&data)?;
        Ok(json)
    }

    pub fn parse_jsontable_list(json: Value) -> Result<Vec<TableInfo>, ParsingError> {
        let mut tables = Vec::new();


        let structure = extract_value_from_json(&json, "structure")?;
        let key_families_object = extract_value_from_json(structure, "keyfamilies")?;
        let keys = extract_array_from_json(key_families_object, "keyfamily")?;
        for key in keys {
            let annotations_object = extract_value_from_json(key, "annotations")?;
            let annotations_array = extract_array_from_json(annotations_object, "annotation")?;
            let mut is_census = false;
            let mut annotation_properties = HashMap::with_capacity(annotations_array.len());
            let id = extract_string_from_json(key, "id")?;
            annotation_properties.insert("id".to_string(), id);
            for annotation in annotations_array {
                let title = extract_string_from_json(annotation, "annotationtitle")?;
                let text = extract_string_from_json(annotation, "annotationtext")?;
                annotation_properties.insert(title, text);
            }
            let table_info = TableInfo::try_from(annotation_properties);
            if let Ok(table_info) = table_info {
                if table_info.geo_level.contains(&"oa".to_string()) {
                    println!("{}", table_info);
                    tables.push(table_info);
                }
            }
        }
        Ok(tables)
        //println!("JSON Data: {:?}", json);
        //println!("{:?}", data);
    }
    pub async fn get_geography_code(&self, id: String) -> Result<(), ParsingError> {
        let mut path = String::from("https://www.nomisweb.co.uk/api/v01/dataset/");
        path.push_str(&id);
        path.push_str("/geography.def.sdmx.json");
        println!("Making request to: {}", path);
        let request = self.client.get(path).send().await?;
        println!("Got response: {:?}", request);
        let data = request.text().await?;
        println!("{}", data);
        /*        for line in data.split("\n") {
                    println!("{}", line);
                }
        */        Ok(())
    }
    pub async fn get_table(&self, id: String, number_of_records: usize, page_size: usize) -> Result<String, ParsingError> {
        let mut path = String::from(NOMIS_API);
        path.push_str(&id);
        path.push_str(".data.csv");
        path.push_str("?geography=");
        path.push_str(ENGLAND_OUTPUT_AREAS_CODE);
        path.push_str("&recordlimit=");
        path.push_str(page_size.to_string().as_str());
        let mut data = String::new();
	let start_time=Instant::now();
        for index in 0..(number_of_records as f64 / page_size as f64).ceil() as usize {
            let mut to_send = path.clone();
            to_send.push_str("&RecordOffset=");
            to_send.push_str((index * page_size).to_string().as_str());
            if index != 0 {
                to_send.push_str("&ExcludeColumnHeadings=true");
            }
            to_send.push_str("&select=");
            to_send.push_str(SELECTED_COLUMNS);
            info!("Making request to: {}", to_send);
            let request = self.client.get(to_send).send().await?;
            debug!("Got response: {:?}", request);
            let new_data = request.text().await?;
            data.push_str(new_data.as_str());
	    info!("Completed request {} in {:?}",index,start_time.elapsed());
        }
        return Ok(data);
    }
    pub fn parse_table<R: std::io::Read>(mut data: csv::Reader<R>) -> Result<HashMap<String, PopulationRecord>, ParsingError> {
        //let mut csv_reader = csv::Reader::from_reader(data.as_bytes());
        let mut output = HashMap::new();

        let mut current_area = String::from("");
        let mut buffer = Vec::new();

        for line in data.deserialize() {
            let record: Result<PreProcessingRecord, csv::Error> = line;
            match record {
                Ok(record) => {
                    if record.geography_name != current_area {
                        if !current_area.is_empty() {
                            let pop_record = PopulationRecord::try_from(buffer);
                            match pop_record {
                                Ok(pop_record) => { output.insert(current_area, pop_record); },
                                Err(e) => { error!("{}",e); }
                            }
                            buffer = Vec::new();
                        }
                        current_area = String::from(&record.geography_name);
                    }
                    buffer.push(record);
                }
                Err(e) => error!("{}",e)
            }
        }
        Ok(output)
    }
    pub async fn read_json(filename: String) -> Result<Value, String> {
        let mut file = File::open(filename).map_err(|e| format!("{:?}", e))?;
        let mut json: Value = serde_json::from_reader(file).map_err(|e| format!("{:?}", e))?;
        Ok(json)
    }
    pub fn write_file(filename: String, data: &String) -> Result<(), String> {
        let mut file = File::create(filename).map_err(|e| format!("{:?}", e))?;
        file.write_all(data.as_bytes()).map_err(|e| format!("{:?}", e))?;
        file.flush();
        Ok(())
    }
}
