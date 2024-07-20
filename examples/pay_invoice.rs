use std::env;

use dotenvy::dotenv;
use strike_rs::pay_ln::PayInvoiceQuoteRequest;
use strike_rs::Strike;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let strike = Strike::new(&env::var("API_KEY").expect("API key not set"), None).unwrap();

    let invoice = "lnbc100n1pnfhjd8pp5vssdjgseqjfs5av4sqymk7ns0u3ldj2904npwue3na2yr0k379kqdq2f38xy6t5wvcqzzsxqrpcgsp58qn6n6f5pj5leuh28f6gz32kgmyzl987htduzatj69nypmdddlxs9qxpqysgqwv48q7ypza0wryu854h9y0ffude4pu857ksu5wa3dt9kn557tsrhx38lzjaece44gfner9rwhsw5cj2e7pt5ckse84t5865m2gczfdsqvtukva".to_string();

    let paymet_quote_request = PayInvoiceQuoteRequest {
        ln_invoice: invoice,
        source_currency: strike_rs::Currency::BTC,
    };
    let quote = strike.payment_quote(paymet_quote_request).await.unwrap();

    println!("{:?}", quote);
    let pay_response = strike.pay_quote(&quote.payment_quote_id).await.unwrap();
    println!("{:?}", pay_response);
}
