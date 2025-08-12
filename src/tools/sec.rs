use bytestack::{
    backend::RedbBackend, database::Database, serialize::RkyvSerializer, table::TableBehavior,
};
use rkyv::{api::low::deserialize, Archive, Deserialize, Serialize};
use sec_lib::{company_names::request_lookup_data, tickers::get_tickers};
use serde::Serialize as serde_Serialize;

#[derive(Deserialize, Serialize, Archive)]
pub struct CompanyInfo {
    cik: u64,
    names: Vec<String>,
    tickers: Vec<String>,
}

struct CompaniesTable {
    key_serializer: RkyvSerializer<u64>,
    value_serializer: RkyvSerializer<CompanyInfo>,
}

impl TableBehavior<u64, CompanyInfo> for CompaniesTable {
    fn table_name(&self) -> &'static str {
        "companies"
    }

    type KeySerializer = RkyvSerializer<u64>;

    type ValueSerializer = RkyvSerializer<CompanyInfo>;

    fn key_serializer(&self) -> &Self::KeySerializer {
        &self.key_serializer
    }

    fn value_serializer(&self) -> &Self::ValueSerializer {
        &self.value_serializer
    }
}

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

    let stack = RedbBackend::new(DB_PATH)?;
    let mut db = Database::new(stack);
    let table_def = CompaniesTable {
        key_serializer: RkyvSerializer::new(),
        value_serializer: RkyvSerializer::new(),
    };
    let mut written = 0u64;
    db.write(|session| {
        let table = session.table(table_def);
        for cik in sorted_ciks {
            // if written % 1000 == 0 {
            //     println!("Written {} records", written);
            // }
            let empty_vec = Vec::new();
            let names = cik_to_names.get(&cik).unwrap_or(&empty_vec);
            let tickers = cik_to_tickers.get(&cik).unwrap_or(&empty_vec);
            let info = CompanyInfo {
                cik,
                names: names.clone(),
                tickers: tickers.clone(),
            };
            table.put(&cik, &info)?;
            written += 1;
        }
        Ok(())
    })?;

    Ok(())
}

fn _foo() {
    let ct = CompaniesTable {
        key_serializer: RkyvSerializer::<u64>::new(),
        value_serializer: RkyvSerializer::<CompanyInfo>::new(),
    };
    let db = get_sec_db();
    let mut count = 0u64;
    let mut _name_chars = 0u64;
    db.read(|session| {
        let table = session.table(ct);
        table.range(0u64, u64::MAX, |_cik, _company| {
            count += 1;
            // name_chars += company
            //     .names
            //     .iter()
            //     .map(|name| name.chars().count() as u64)
            //     .sum::<u64>();
            // Ok(())
        })?;
        Ok(())
    })
    .unwrap();
    println!("Total companies processed: {}", count);
    println!("Total name characters processed: {}", _name_chars);
}

fn get_sec_db() -> Database<RedbBackend> {
    let stack = RedbBackend::new(DB_PATH).unwrap();
    Database::new(stack)
}

#[derive(serde_Serialize)]
pub struct CikInfoOut {
    cik: u64,
    names: Vec<String>,
    tickers: Vec<String>,
}
use rkyv::rancor::Error;
pub fn get_cik_info(cik: u64) -> Result<Option<CikInfoOut>, String> {
    let table_def = CompaniesTable {
        key_serializer: RkyvSerializer::new(),
        value_serializer: RkyvSerializer::new(),
    };
    let mut info = None;
    get_sec_db()
        .read(|session| {
            let table = session.table(table_def);
            table.with(&cik, |company| {
                info = Some(CikInfoOut {
                    cik: cik,
                    names: deserialize::<Vec<String>, Error>(&company.names).unwrap(),
                    tickers: deserialize::<Vec<String>, Error>(&company.tickers).unwrap(),
                });
                Ok(())
            })
        })
        .unwrap();
    Ok(info)
}

pub fn get_company_substring_search(substring: &str) -> Result<Vec<CikInfoOut>, String> {
    let table_def = CompaniesTable {
        key_serializer: RkyvSerializer::new(),
        value_serializer: RkyvSerializer::new(),
    };
    let mut results = Vec::new();
    let all_caps = substring.to_uppercase();
    get_sec_db()
        .read(|session| {
            let table = session.table(table_def);
            table.range(0u64, u64::MAX, |cik, company| {
                let mut is_substring = false;
                for name in company.names.iter() {
                    // println!("name: {}", name);
                    if name.contains(&all_caps) {
                        is_substring = true;
                        break;
                    }
                }

                if is_substring {
                    results.push(CikInfoOut {
                        cik: deserialize::<u64, Error>(cik).unwrap(),
                        names: deserialize::<Vec<String>, Error>(&company.names).unwrap(),
                        tickers: deserialize::<Vec<String>, Error>(&company.tickers).unwrap(),
                    });
                }
                // Ok(())
            })
        })
        .unwrap();
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sec() {
        // sec().await;
        let start = std::time::Instant::now();
        _foo();
        let duration = start.elapsed();
        println!("Test completed in {:?}", duration);
    }

    #[tokio::test]
    async fn test_get_cik_info() {
        let cik = 123456;
        let result = get_cik_info(cik);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.cik, cik);
    }

    #[tokio::test]
    async fn test_get_company_substring_search() {
        let substring = "GOOGLE";
        let result = get_company_substring_search(substring);
        assert!(result.is_ok());
        let companies = result.unwrap();
        assert!(!companies.is_empty());
        for company in companies {
            assert!(company.names.iter().any(|name| name.contains(substring)));
            println!("Company CIK: {}", company.cik);
            println!("Names: {:?}", company.names);
            println!("Tickers: {:?}", company.tickers);
        }
    }
}
