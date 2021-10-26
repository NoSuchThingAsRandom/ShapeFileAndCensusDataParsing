use std::fs;
use walkdir::WalkDir;

// Data Source: https://www.nomisweb.co.uk/census/2011
fn table_type_code_to_string(code: &str) -> Result<&str, String> {
    if code.len() != 2 {
        return Err(format!("Table Code is not the correct amount of characters {}", code.len()));
    }
    return match code {
        "QS" => { Ok("Quick Statistics") }
        "KS" => { Ok("Key Statistics") }
        "DC" => { Ok("Detailed Characteristics") }
        "LC" => { Ok("Local Characteristics") }
        "WD" => { Ok("Workday Population") }
        "WP" => { Ok("Workplace Population Tables") }
        "CT" => { Ok("Commissioned Tables") }
        _ => {
            Err(format!("Unknown table code {}", code))
        }
    };
}

fn country_code_to_string(code: &str) -> Result<&str, String> {
    if code.len() != 2 {
        return Err(format!("Country Code is not the correct amount of characters {} ", code.len()));
    }
    return match code {
        "EW" => { Ok("England & Wales") }
        "WA" => { Ok("Wales") }
        _ => {
            Err(format!("Unknown country code: {}", code))
        }
    };
}


fn geography_code_to_string(code: &str) -> Result<&str, String> {
    if code.len() != 1 && code.len() != 2 {
        return Err(format!("Geography Code is not the correct amount of characters {}", code.len()));
    }
    return match code {
        "r" => { Ok("Region") }
        "la" => { Ok("Merged Wards") }
        "ls" => { Ok("Lower Level Super Output Area") }
        _ => {
            Ok("")
        }
    };
}

fn parse_filename(name: &str) -> Result<String, String> {
    let name = name.split("/").last().unwrap();
    if name.len() < 16 || name.len() > 20 {
        return Err(format!("Name is not the correct amount of characters {} {}", name, name.len()));
    }
    if !name.contains("DATA.CSV") {
        return Err(format!("Invalid file"));
    }

    let mut new_name = String::new();
    new_name += table_type_code_to_string(&name[0..2])?;
    new_name += " - ";
    new_name += &name[2..6];
    new_name += " - ";
    new_name += country_code_to_string(&name[6..8])?;
    new_name += " - ";
    new_name += geography_code_to_string(&name[6..8])?;
    new_name += " - ";
    new_name += name;
    Ok(new_name)
}
