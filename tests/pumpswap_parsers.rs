use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::fs;
use std::path::Path;
use std::str::FromStr;

async fn fetch_transaction_fixture(
    signature: &str,
    fixture_path: &Path,
) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
    if fixture_path.exists() {
        let data = fs::read_to_string(fixture_path)?;
        return Ok(serde_json::from_str(&data)?);
    }

    let client = RpcClient::new_with_commitment(
        "https://api.mainnet-beta.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    let tx = client
        .get_transaction_with_config(
            &Signature::from_str(signature)?,
            solana_client::rpc_config::RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Base64),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )
        .await?;

    fs::write(fixture_path, serde_json::to_string_pretty(&tx)?)?;
    Ok(tx)
}

#[tokio::test]
async fn test_pumpswap_cpi_transaction_parsing() -> Result<()> {
    use solana_streamer_sdk::streaming::event_parser::{
        protocols::pumpswap::PumpSwapBuyEvent, EventParserFactory, Protocol,
    };
    use solana_transaction_status::EncodedTransactionWithStatusMeta;

    // Transaction with PumpSwap called via CPI (currently failing to parse correctly)
    let signature = "56RbkzmAEtd88ZeiBigh41kPThpoFqZoxj9tULQJe7xRBAcdRYxREuNBRUW5f2jJASZ81aNhxe8EBej258q76AuH";
    let fixture_path = Path::new("tests/fixtures/pumpswap_cpi_tx.json");
    let tx = fetch_transaction_fixture(signature, fixture_path).await?;

    let parser = EventParserFactory::create_parser(Protocol::PumpSwap);
    let encoded_tx = EncodedTransactionWithStatusMeta {
        transaction: tx.transaction.transaction.clone(),
        meta: tx.transaction.meta.clone(),
        version: tx.transaction.version,
    };

    let events = parser
        .parse_transaction(
            encoded_tx,
            signature,
            Some(tx.slot),
            tx.block_time.map(|bt| prost_types::Timestamp {
                seconds: bt / 1000,
                nanos: ((bt % 1000) * 1_000_000) as i32,
            }),
            None,
        )
        .await?;

    // The transaction should parse at least one buy event
    assert!(!events.is_empty(), "Should find at least one event");

    let buy_event = events
        .iter()
        .find_map(|e| e.as_any().downcast_ref::<PumpSwapBuyEvent>())
        .expect("Should find at least one PumpSwap buy event");

    // Expected values from the transaction data you provided
    assert_eq!(buy_event.timestamp, 1753789195);
    assert_eq!(buy_event.base_amount_out, 5544465819);
    assert_eq!(buy_event.max_quote_amount_in, 963427853);
    assert_eq!(buy_event.user_base_token_reserves, 1);
    assert_eq!(buy_event.user_quote_token_reserves, 2856323143);
    assert_eq!(buy_event.pool_base_token_reserves, 30819310537017);
    assert_eq!(buy_event.pool_quote_token_reserves, 5338304399191);
    assert_eq!(buy_event.quote_amount_in, 960546212);
    assert_eq!(buy_event.lp_fee_basis_points, 20);
    assert_eq!(buy_event.lp_fee, 1921093);
    assert_eq!(buy_event.protocol_fee_basis_points, 5);
    assert_eq!(buy_event.protocol_fee, 480274);
    assert_eq!(buy_event.quote_amount_in_with_lp_fee, 962467305);
    assert_eq!(buy_event.user_quote_amount_in, 963427853);
    
    // Account assertions
    assert_eq!(buy_event.pool.to_string(), "4w2cysotX6czaUGmmWg13hDpY4QEMG2CzeKYEQyK9Ama");
    assert_eq!(buy_event.user.to_string(), "6TYDxGmVxkBPBmEfnmLXx6jVff9LknsjRHqdTjVyZmG8");
    assert_eq!(buy_event.user_base_token_account.to_string(), "4yFkSKTzpbDhFpFEHiooKg9XmoNbQGfbe4ZPxK7CRtHB");
    assert_eq!(buy_event.user_quote_token_account.to_string(), "13Ma7KdfbBrF8NmDBnVTkwny7tjrZxYHCfn6a3FFbRUa");
    assert_eq!(buy_event.protocol_fee_recipient.to_string(), "62qc2CNXwrYqQScmEdiZFFAnJR262PxWEuNQtxfafNgV");
    assert_eq!(buy_event.protocol_fee_recipient_token_account.to_string(), "94qWNrtmfn42h3ZjUZwWvK1MEo9uVmmrBPd2hpNjYDjb");
    assert_eq!(buy_event.coin_creator.to_string(), "hQmkJVU4iWgxCHZvVACpVPa1KN28oGxXpRmcYzYNwck");
    assert_eq!(buy_event.coin_creator_fee_basis_points, 5);
    assert_eq!(buy_event.coin_creator_fee, 480274);

    Ok(())
}