use super::*;
use niffyinsure::types::{PolicyType, RegionTier};
use soroban_sdk::Env;
use std::collections::HashMap;

#[test]
fn premium_pure_table_tests() {
    let env = Env::default();

    // Spreadsheet export parity vectors.
    // Format: JSON/CSV with cols: policy_type, region, age, risk_score, expected_total_stroops, expected_line_items (optional)
    // Docs: Col A PolicyType -> PremiumFactors::new type_f
    // Col B RegionTier -> region_f
    // Col C age -> age_f logic
    // Col D risk_score -> risk_f
    // Col E expected = compute_premium_pure

    let test_cases_json = r#"
[
  {
    "policy_type": "Auto",
    "region": "Medium",
    "age": 30,
    "risk_score": 6,
    "expected_total": 49000000
  },
  {
    "policy_type": "Health",
    "region": "High",
    "age": 22,
    "risk_score": 3,
    "expected_total": 58400000
  },
  {
    "policy_type": "Property",
    "region": "Low",
    "age": 65,
    "risk_score": 1,
    "expected_total": 30100000
  },
  {
    "policy_type": "Auto",
    "region": "High",
    "age": 18,
    "risk_score": 10,
    "expected_total": 63800000
  }
]
"#;

    let test_cases: Vec<HashMap<String, serde_json::Value>> = serde_json::from_str(test_cases_json).unwrap();

    for case in test_cases {
        let policy_type = match case["policy_type"].as_str().unwrap() {
            "Auto" => PolicyType::Auto,
            "Health" => PolicyType::Health,
            "Property" => PolicyType::Property,
            _ => panic!("invalid policy_type"),
        };
        let region = match case["region"].as_str().unwrap() {
            "Low" => RegionTier::Low,
            "Medium" => RegionTier::Medium,
            "High" => RegionTier::High,
            _ => panic!("invalid region"),
        };
        let age: u32 = case["age"].as_u64().unwrap() as u32;
        let risk_score: u32 = case["risk_score"].as_u64().unwrap() as u32;
        let expected_total: i128 = case["expected_total"].as_str().unwrap().parse().unwrap();

        let client = NiffyInsureClient::new(&env, &contract_id);  // assume initialized, but pure no need

        let factors = premium_pure::PremiumFactors::new(&policy_type, &region, age, risk_score).unwrap();
        let computed = premium_pure::compute_premium_pure(&factors).unwrap();

        assert_eq!(computed, expected_total, "mismatch for {:?}", case);
    }
}

