use crate::storage::{Recreate, Storage, StorageConfig};
use axum::http::request;
use sec_lib::company_names::request_lookup_data;

// pub fn get_cik(ticker: &str) -> Result<String, String> {
//     // let path = "./ignore/ticker.txt";
//     // let cik_query = CIKQuery::new(Some(path)).map_err(|e| e.to_string())?;
//     // cik_query.get_cik(ticker).await.map_err(|e| e.to_string())
// }

pub async fn sec() {
    // sec_lib::company_names::request_lookup_data()
    //     .await
    //     .expect("Failed to request lookup data");

    match request_lookup_data().await {
        Ok(response) => {
            let names = sec_lib::company_names::consume_response(response).await;
            let config = StorageConfig::new("sec.db".to_string(), Recreate::IfMissing);
            let db = Storage::new(config).expect("Failed to initialize storage");
            println!("Names: {:?}", names);
        }
        Err(e) => {
            eprintln!("Error fetching lookup data: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sec() {
        sec().await;
    }
}
