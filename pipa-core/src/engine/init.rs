use crate::logging::init::init_logging;
use crate::logging::ledger::ensure_ledger_key_exists;
use std::error::Error;
use std::fs;
use std::path::Path;

// Import necessary crates for CSV generation and fake data
use csv::Writer;
use fake::{Dummy, Fake, Faker};
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::{FirstName, LastName};
use serde::Serialize;

/// Represents a single customer record for our sample CSV.
/// The `Serialize` trait allows this struct to be written to CSV format automatically.
/// The `Fake` trait allows us to generate random instances of this struct.
#[derive(Serialize, Dummy)]
struct Customer {
    id: i64,
    // 3. The attribute is now `dummy` and it takes a string faker.
    #[dummy(faker = "FirstName()")]
    first_name: String,
    #[dummy(faker = "LastName()")]
    last_name: String,
    #[dummy(faker = "SafeEmail()")]
    email: String,
}

/// Creates a sample customers.csv file with 30 rows of fake data.
fn create_sample_csv(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;

    for i in 1..=30 {
        // The generation logic itself remains the same
        let mut customer: Customer = Faker.fake();
        customer.id = i;
        
        wtr.serialize(customer)?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn init_project() -> Result<String, Box<dyn Error>> {
    init_logging();
    ensure_ledger_key_exists();

    let mut actions = Vec::new();

    // --- Create Directories ---
    let contracts_dir = Path::new("contracts");
    fs::create_dir_all(contracts_dir)?;

    let data_dir = Path::new("data");
    if !data_dir.exists() {
        fs::create_dir_all(data_dir)?;
        actions.push("Created data directory");
    }

    if !Path::new(".env").exists() {
        let env_content = r#"# .env file for project-specific environment variables

S3_ACCESS_KEY=<access_key>
S3_SECRET_KEY=<secret_key>

#Currently this uses the key from the storage account. If you use in production remember to rotate the key based on your companies policy
AZURE_STORAGE_CONNECTION_STRING=DefaultEndpointsProtocol=https;AccountName=<account_name>;AccountKey=<account_key>;EndpointSuffix=core.windows.net

GCP_SERVICE_ACCOUNT_KEY='{"type": "service_account","project_id": <your_project_id>,"private_key_id": <your_private_key_id>,"private_key": "-----BEGIN PRIVATE KEY-----<The actual key from the file, this will be very long>-----END PRIVATE KEY-----\n","client_email": <user@project.iam.gserviceaccount.com>,"client_id": <your_client_id>,"auth_uri": "https://accounts.google.com/o/oauth2/auth","token_uri": "https://oauth2.googleapis.com/token","auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs","client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/<email>","universe_domain": "googleapis.com"}'
"#;
        fs::write(".env", env_content)?;
        actions.push("Created .env file");
    }

    if !Path::new("profiles.toml").exists() {
        let profile_content = r#"# Pipe Audit profiles.toml example
[s3_test]
provider   = "s3"                       
endpoint   = "http://developyr.local:9000"    
region     = "us-east-1"                
access_key = "${S3_ACCESS_KEY}"     
secret_key = "${S3_SECRET_KEY}"
path_style = true
use_ssl = false

[azure_test]
provider = "azure"
connection_string = "${AZURE_STORAGE_CONNECTION_STRING}"

[gcs_test]
provider = "gcs"
service_account_json = "${GCP_SERVICE_ACCOUNT_KEY}"
"#;
        fs::write("profiles.toml", profile_content)?;
        actions.push("Created profiles.toml");
    }

    // Use `join` to build paths robustly for any OS.
    let contract_path = contracts_dir.join("example.toml");
    if !contract_path.exists() {
        let contract_content = r#"# contracts/example.toml
[contract]
name = "customers"
version = "0.1.1"
tags = ["pii", "critical"]

[file]
validation = [
  { rule = "row_count", min = 0, max = 20}
]

[[columns]]
name = "id"
validation = [
  { rule = "not_null" },
  { rule = "unique" }
]

[[columns]]
name = "email
validation = [
  { rule = "pattern", pattern = "^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$" }
]

[source]
type = "local"
location = "data/customers.csv"
profile = "local"

"#;
        fs::write(&contract_path, contract_content)?;
        actions.push("Created contracts/example.toml");
    }

    let customers_csv_path = data_dir.join("customers.csv");
    if !customers_csv_path.exists() {
        create_sample_csv(&customers_csv_path)?;
        actions.push("Created sample data/customers.csv");
    }

    if actions.is_empty() {
        Ok("Project already initialized. No changes were made.".to_string())
    } else {
        Ok(format!(
            "Successfully initialized project. Actions: {}",
            actions.join(", ")
        ))
    }
}
