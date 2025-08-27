// https://iptoasn.com/data/ip2asn-combined.tsv.gz

use flate2::read::GzDecoder;
use reqwest::blocking::get;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::io::{ BufRead, BufReader };

#[derive(Serialize)]
struct ApiResponse {
    result: Option<HashMap<String, String>>,
    error: Option<String>,
}

const HEADERS: [&str; 4] = ["IP", "Number", "Country", "Host"];
const TSV_GZ_URL: &str = "https://example.com/data.tsv.gz"; // Replace with your actual URL

fn main() {
    let query = env::var("QUERY_STRING").unwrap_or_default();
    let search_term = get_query_param(&query, "search_term");

    let result = match search_term {
        Some(term) =>
            match find_row_in_gzipped_tsv(TSV_GZ_URL, &term) {
                Ok(Some(row)) =>
                    ApiResponse {
                        result: Some(row),
                        error: None,
                    },
                Ok(None) =>
                    ApiResponse {
                        result: None,
                        error: Some("No matching row found.".to_string()),
                    },
                Err(e) =>
                    ApiResponse {
                        result: None,
                        error: Some(format!("Error: {}", e)),
                    },
            }
        None =>
            ApiResponse {
                result: None,
                error: Some("Missing 'search_term' query parameter.".to_string()),
            },
    };

    let json = serde_json::to_string(&result).unwrap();
    println!("Content-Type: application/json\n");
    println!("{}", json);
}

fn get_query_param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        let k = parts.next()?;
        let v = parts.next()?;
        if k == key {
            Some(v.to_string())
        } else {
            None
        }
    })
}

fn find_row_in_gzipped_tsv(
    url: &str,
    search: &str
) -> Result<Option<HashMap<String, String>>, Box<dyn std::error::Error>> {
    let response = get(url)?;
    let bytes = response.bytes()?;

    let decoder = GzDecoder::new(&bytes[..]);
    let reader = BufReader::new(decoder);

    for line_result in reader.lines() {
        let line = line_result?;
        let values: Vec<String> = line
            .split('\t')
            .map(|s| s.to_string())
            .collect();

        if values.iter().any(|v| v.contains(search)) {
            if values.len() != HEADERS.len() {
                return Err(
                    format!(
                        "Expected {} columns, but got {}: {:?}",
                        HEADERS.len(),
                        values.len(),
                        line
                    ).into()
                );
            }

            let row: HashMap<String, String> = HEADERS.iter()
                .cloned()
                .map(String::from)
                .zip(values)
                .collect();

            return Ok(Some(row));
        }
    }

    Ok(None)
}
