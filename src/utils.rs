use crate::structs::{self, ContestResult};
use crate::ApiResponse;
use dotenv::dotenv;
use reqwest::blocking::get;
use sha2::{Digest, Sha512};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::time::{SystemTime, UNIX_EPOCH};

use structs::{Row, Battle};

pub(crate) fn calculate_scores(contest_rows: &mut Vec<Row>) {
    let n: f32 = contest_rows.len() as f32;
    let mx_slv: f32 = contest_rows.iter().map(|r| r.points).max().unwrap() as f32;

    for (rank, sc, pts) in contest_rows
        .iter_mut()
        .map(|r| (r.rank, &mut r.score, r.points))
    {
        *sc = 200.0;
        *sc *= (n + 1.0 - (rank as f32)) / n;
        *sc *= (pts as f32) / (mx_slv as f32);
    }
}

fn create_signed_request(
    method: &str,
    mut params: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    // Get current timestamp (seconds since Unix epoch)
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let time_str = time.to_string();

    // Get last 6 digits of timestamp
    let rand = &time_str[time_str.len().saturating_sub(6)..];

    dotenv().expect("failed to load .env file");
    let api_key = std::env::var("APIKEY").expect("APIKEY var must be set");
    let api_secret = std::env::var("APISECRET").expect("APISECRET var must be set");

    // Add API key and timestamp to parameters
    params.insert("apiKey".to_string(), api_key.to_string());
    params.insert("time".to_string(), time_str.clone());

    // Sort parameters using BTreeMap (already sorted by key)
    let param_str = params
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<String>>()
        .join("&");

    // Create string to hash
    let to_hash = format!("{}/{}?{}#{}", rand, method, param_str, api_secret);

    // Compute SHA-512 hash
    let mut hasher = Sha512::new();
    hasher.update(to_hash.as_bytes());
    let hash_result = hasher.finalize();

    // Convert hash to hex string
    let api_sig_hex = hex::encode(hash_result);

    // Return updated parameters with signature
    let mut signed_params = params.clone();
    signed_params.insert("apiSig".to_string(), format!("{}{}", rand, api_sig_hex));

    signed_params
}

fn fetch_json_data(group_code: &str, contest_id: &str) -> String {
    let method = "contest.standings".to_string();

    // Create parameters map
    let mut params = BTreeMap::new();
    params.insert("contestId".to_string(), contest_id.to_string());
    params.insert("from".to_string(), "1".to_string());
    params.insert("count".to_string(), "100000".to_string());
    params.insert("groupCode".to_string(), group_code.to_string());

    // Generate signed parameters
    let signed_params = create_signed_request(&method, params);

    // Convert signed parameters into query string
    let query_string = signed_params
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<String>>()
        .join("&");

    // Construct final URL
    let url = format!("https://codeforces.com/api/{}?{}", method, query_string);
    get(url)
        .expect("Failed to fetch API")
        .text()
        .expect("Failed to read response")
}

pub(crate) fn get_all_contests() -> Vec<ContestResult> {
    /* yeah should not hardcode it but for now it is what it is */
    let filename = "data/battles.json";
    let file = File::open(filename).expect(&format!("failed to open {}", filename));
    let reader = BufReader::new(file);

    let battles: Vec<Battle> = serde_json::from_reader(reader).expect("failed to parse json");
    let mut res: Vec<ContestResult> = vec![];
    for b in battles {
        let json_str = fetch_json_data(&b.group_code, &b.contest_id);
        let response: ApiResponse =
            serde_json::from_str(&json_str).expect("failed to parse the json response");
        res.push(response.result);
    }

    res
}

pub(crate) fn sort_rows_by_team(contest_rows: &Vec<Row>) -> HashMap<String, Vec<Row>> {
    /* yeah should not hardcode it but for now it is what it is */
    let filename = "data/teams.json";
    let file = File::open(filename).expect(&format!("failed to open {}", filename));
    let reader = BufReader::new(file);
    let mapping: HashMap<String, String> =
        serde_json::from_reader(reader).expect("failed to parse json");

    let filename = "data/leaders.json";
    let file = File::open(filename).expect(&format!("failed to open {}", filename));
    let reader = BufReader::new(file);
    let leader_map: HashMap<String, String> =
        serde_json::from_reader(reader).expect("failed to parse json");

    let mut res = HashMap::<String, Vec<Row>>::new();
    for r in contest_rows {
        let key = mapping
            .get(&r.party.members.first().unwrap().handle)
            .expect("handle not found in team mapping");
        let leader = leader_map
            .get(key)
            .expect("leader not found in leader mapping");
        res.entry(leader.clone())
            .or_insert_with(Vec::new)
            .push(r.clone());
    }

    res
}