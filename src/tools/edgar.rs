use std::path::Path;

use sec_edgar::{
    edgar::{edgar_client, get_feed_entries, get_feed_entry_content},
    edgar_query::{
        cik_query::CIKQuery,
        edgar_query_builder::{BuilderInput, EdgarQueryBuilder},
        filing::FilingTypeOption::_10Q,
    },
};

async fn _some_func() -> String {
    let ticker = "1509607";
    // To save yourself a trip, you can store the file locally and query it instead.
    // The file can be downloaded from [here](https://www.sec.gov/include/ticker.txt).
    // let cik_query = CIKQuery::new(Some("./ignore/ticker.txt"))

    let path = "/home/keith/code/keipes-ai-workspace/cik-lookup-data.txt";

    CIKQuery::new(Some(path))
        .unwrap()
        .get_cik(ticker)
        .await
        .unwrap();
    assert!(
        Path::new(path).is_file(),
        "Ticker file does not exist at {}",
        path
    );
    let cik_query = CIKQuery::new(Some(path))
        .unwrap()
        .get_cik(ticker)
        .await
        .unwrap();
    let query = EdgarQueryBuilder::new(&cik_query)
        .set_filing_type(BuilderInput::TypeTInput(_10Q))
        .build()
        .unwrap();
    let entries = get_feed_entries(edgar_client().unwrap(), query)
        .await
        .unwrap();
    let filing_type: String = get_feed_entry_content(entries.first().unwrap())
        .unwrap()
        .filing_type
        .value;
    filing_type
}

// unit tests
// write tests below here
#[cfg(test)]
mod tests {
    use dotenvy::dotenv;

    use super::*;

    #[tokio::test]
    async fn test_moka_cache() {
        dotenv().ok();

        // SET USER_AGENT env var
        std::env::set_var("USER_AGENT", "Keith Smith <keithpsmith@gmail.com>");

        let result = _some_func().await;
        assert_eq!(result, "expected_value");
        println!("Some func {}", result);
    }
}
