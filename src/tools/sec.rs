use crate::{
    generated::{self, CikValue},
    storage::{Recreate, Storage, StorageConfig},
};
use axum::http::request;
use bytes::Bytes;
use flatbuffers::FlatBufferBuilder;
use sec_lib::{company_names::request_lookup_data, tickers::get_tickers};

// pub fn get_cik(ticker: &str) -> Result<String, String> {
//     // let path = "./ignore/ticker.txt";
//     // let cik_query = CIKQuery::new(Some(path)).map_err(|e| e.to_string())?;
//     // cik_query.get_cik(ticker).await.map_err(|e| e.to_string())
// }

static DB_PATH: &str = "cik.db";

pub async fn sec() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = get_tickers().await?;
    let cik_to_tickers = sec_lib::tickers::parse_tickers(tickers).await;
    tracing::info!("Parsed tickers: {:?}", cik_to_tickers);
    let lookup_data = request_lookup_data().await?;
    let cik_to_names = sec_lib::company_names::consume_response(lookup_data).await;
    tracing::info!("Parsed company names: {:?}", cik_to_names);
    let mut all_ciks_set = cik_to_names
        .keys()
        .cloned()
        .collect::<std::collections::HashSet<_>>();
    all_ciks_set.extend(cik_to_tickers.keys().cloned());
    tracing::info!("# CIKs: {:?}", all_ciks_set.len());
    let mut sorted_ciks = all_ciks_set.into_iter().collect::<Vec<_>>();
    sorted_ciks.sort_unstable();
    let db = Storage::new(StorageConfig::new(DB_PATH.into(), Recreate::IfMissing))
        .expect("Failed to initialize storage");

    let mut fbb = FlatBufferBuilder::new();
    db.write(crate::storage::CIK_TABLE, |table| {
        fbb.reset();
        for cik in sorted_ciks {
            let empty_vec = Vec::new();
            let names = cik_to_names.get(&cik).unwrap_or(&empty_vec);
            let name_offsets: Vec<_> = names.iter().map(|name| fbb.create_string(name)).collect();
            let flatnames = fbb.create_vector(&name_offsets);

            let tickers = cik_to_tickers.get(&cik).unwrap_or(&empty_vec);
            let ticker_offsets: Vec<_> = tickers
                .iter()
                .map(|ticker| fbb.create_string(ticker))
                .collect();
            let st = fbb.create_vector(&ticker_offsets);

            let mut value = crate::generated::CikValueBuilder::new(&mut fbb);
            value.add_tickers(st);
            value.add_company_names(flatnames);
            let serialized = value.finish();
            table.insert(&cik, &serialized);
        }
        Ok(())
    });
    Ok(())
}

fn write_all(cik_values: Vec<CikValue>) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sec() {
        sec().await;
    }
}
