use std::env;

use dotenvy::dotenv;
use strike_rs::{Amount, InvoiceRequest, Strike};

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let strike = Strike::new(&env::var("API_KEY").expect("API key not set"), None).unwrap();

    let invoice_request = InvoiceRequest {
        correlation_id: None,
        description: None,
        amount: Amount::from_sats(100),
    };

    let create_invoice = strike.create_invoice(invoice_request).await.unwrap();
    println!("{:?}", create_invoice);

    let invoice = strike
        .find_invoice(&create_invoice.invoice_id.clone())
        .await
        .unwrap();
    println!("{:?}", invoice);

    let quote = strike
        .invoice_quote(&create_invoice.invoice_id.clone())
        .await
        .unwrap();

    println!("{:?}", quote);
}
