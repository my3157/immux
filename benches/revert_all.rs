use std::collections::HashMap;

use immuxsys::constants as Constants;
use immuxsys::storage::chain_height::ChainHeight;
use immuxsys::storage::executor::grouping_label::GroupingLabel;
use immuxsys::storage::executor::unit_content::UnitContent;
use immuxsys_client::http_client::ImmuxDBHttpClient;
use immuxsys_client::ImmuxDBClient;
use immuxsys_dev_utils::data_models::census90::CensusEntry;
use immuxsys_dev_utils::dev_utils::{
    csv_to_json_table, launch_test_db_servers, measure_single_operation, read_usize_from_arguments,
};

fn main() {
    let port = 4403;
    let bench_name = "bench_revert_all";
    let row_limit = read_usize_from_arguments(1).unwrap_or(50_000);
    let verify_correctness = read_usize_from_arguments(3).unwrap_or(0) > 0;

    println!(
        "\nExecuting bench {}, with tables truncated at row {}",
        bench_name, row_limit
    );

    launch_test_db_servers("bench_revert_all", Some(port), None).unwrap();

    let grouping = GroupingLabel::from("census90");
    let table = csv_to_json_table::<CensusEntry>(
        "dev_utils/src/data_models/data-raw/census90.txt",
        &grouping.to_string(),
        b',',
        row_limit,
    )
    .unwrap();

    let host = &format!("{}:{}", Constants::SERVER_END_POINT, port);
    let client = ImmuxDBHttpClient::new(host).unwrap();

    for (unit_key, content) in table.iter() {
        client.set_unit(&grouping, &unit_key, &content).unwrap();
    }

    let target_heights = [
        ChainHeight::new(1),
        ChainHeight::new(5),
        ChainHeight::new(10),
        ChainHeight::new(50),
        ChainHeight::new(100),
        ChainHeight::new(500),
        ChainHeight::new(1000),
        ChainHeight::new(5000),
        ChainHeight::new(10000),
        ChainHeight::new(30000),
    ];

    for target_height in target_heights.iter() {
        let total_time = measure_single_operation(
            |height| client.revert_all(height).map_err(|err| err.into()),
            &target_height,
        )
        .unwrap();

        println!(
            "took {}ms to execute {} operation to height {}.",
            total_time,
            "revert_all",
            target_height.as_u64()
        );

        if verify_correctness {
            println!("Verifying correctness.");
            let mut expected_table = HashMap::new();

            for (unit_key, content) in &table[0..target_height.as_u64() as usize + 1] {
                expected_table.insert(unit_key, content);
            }

            for (unit_key, _content) in table.iter() {
                if expected_table.contains_key(unit_key) {
                    let (code, actual_output) = client.get_by_key(&grouping, &unit_key).unwrap();
                    let expected_output = expected_table.get(&unit_key).unwrap().to_string();
                    assert_eq!(code, 200);
                    assert_eq!(
                        UnitContent::from(expected_output.as_str()),
                        UnitContent::from(actual_output.as_str())
                    );
                } else {
                    let (code, actual_output) = client.get_by_key(&grouping, &unit_key).unwrap();
                    assert_eq!(code, 200);
                    assert_eq!(actual_output, "Nil");
                }
            }
            println!("Verifying correctness finished");
        }
    }
}
