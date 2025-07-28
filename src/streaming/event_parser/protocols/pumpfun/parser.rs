use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::streaming::event_parser::{
    common::{EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::pumpfun::{discriminators, PumpFunCreateTokenEvent, PumpFunTradeEvent},
};
use regex::Regex;

/// PumpFun程序ID
pub const PUMPFUN_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

/// PumpFun事件解析器
pub struct PumpFunEventParser {
    inner: GenericEventParser,
}

impl PumpFunEventParser {
    pub fn new() -> Self {
        // 配置所有事件类型
        let configs = vec![
            GenericEventParseConfig {
                inner_instruction_discriminator: discriminators::CREATE_TOKEN_EVENT,
                instruction_discriminator: discriminators::CREATE_TOKEN_IX,
                event_type: EventType::PumpFunCreateToken,
                inner_instruction_parser: Self::parse_create_token_inner_instruction,
                instruction_parser: Self::parse_create_token_instruction,
            },
            GenericEventParseConfig {
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::BUY_IX,
                event_type: EventType::PumpFunBuy,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_buy_instruction,
            },
            GenericEventParseConfig {
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::SELL_IX,
                event_type: EventType::PumpFunSell,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_sell_instruction,
            },
            GenericEventParseConfig {
                inner_instruction_discriminator: "",
                instruction_discriminator: &[0; 8],
                event_type: EventType::Unknown,
                inner_instruction_parser: Self::parse_log_inner_instruction,
                instruction_parser: Self::parse_log_instruction,
            },
        ];

        let inner = GenericEventParser::new(PUMPFUN_PROGRAM_ID, ProtocolType::PumpFun, configs);

        Self { inner }
    }

    /// 解析创建代币日志事件
    fn parse_create_token_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
        _log_messages: &Option<Vec<String>>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if let Ok(event) = borsh::from_slice::<PumpFunCreateTokenEvent>(data) {
            let mut metadata = metadata;
            metadata.set_id(format!(
                "{}-{}-{}-{}",
                metadata.signature,
                event.name,
                event.symbol,
                event.mint.to_string()
            ));
            Some(Box::new(PumpFunCreateTokenEvent {
                metadata: metadata,
                ..event
            }))
        } else {
            None
        }
    }

    /// 解析交易事件
    fn parse_trade_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
        _log_messages: &Option<Vec<String>>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if let Ok(event) = borsh::from_slice::<PumpFunTradeEvent>(data) {
            let mut metadata = metadata;
            metadata.set_id(format!(
                "{}-{}-{}-{}",
                metadata.signature,
                event.mint.to_string(),
                event.user.to_string(),
                event.is_buy.to_string()
            ));
            Some(Box::new(PumpFunTradeEvent {
                metadata: metadata,
                ..event
            }))
        } else {
            None
        }
    }

    /// 解析创建代币指令事件
    fn parse_create_token_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
        _log_messages: &Option<Vec<String>>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 11 {
            return None;
        }
        let mut offset = 0;
        let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let name = String::from_utf8_lossy(&data[offset..offset + name_len]);
        offset += name_len;
        let symbol_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let symbol = String::from_utf8_lossy(&data[offset..offset + symbol_len]);
        offset += symbol_len;
        let uri_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let uri = String::from_utf8_lossy(&data[offset..offset + uri_len]);
        offset += uri_len;
        let creator = if offset + 32 <= data.len() {
            Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?)
        } else {
            Pubkey::default()
        };

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature,
            name,
            symbol,
            accounts[0].to_string()
        ));

        Some(Box::new(PumpFunCreateTokenEvent {
            metadata,
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: uri.to_string(),
            creator,
            mint: accounts[0],
            mint_authority: accounts[1],
            bonding_curve: accounts[2],
            associated_bonding_curve: accounts[3],
            user: accounts[7],
            ..Default::default()
        }))
    }

    // 解析买入指令事件
    fn parse_buy_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
        _log_messages: &Option<Vec<String>>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 11 {
            return None;
        }
        let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let max_sol_cost = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature,
            accounts[2].to_string(),
            accounts[6].to_string(),
            true.to_string()
        ));
        Some(Box::new(PumpFunTradeEvent {
            metadata,
            fee_recipient: accounts[1],
            mint: accounts[2],
            bonding_curve: accounts[3],
            associated_bonding_curve: accounts[4],
            associated_user: accounts[5],
            user: accounts[6],
            creator_vault: accounts[8],
            max_sol_cost,
            amount,
            is_buy: true,
            ..Default::default()
        }))
    }

    // 解析卖出指令事件
    fn parse_sell_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
        _log_messages: &Option<Vec<String>>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 11 {
            return None;
        }
        let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let min_sol_output = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature,
            accounts[2].to_string(),
            accounts[6].to_string(),
            false.to_string()
        ));
        Some(Box::new(PumpFunTradeEvent {
            metadata,
            fee_recipient: accounts[1],
            mint: accounts[2],
            bonding_curve: accounts[3],
            associated_bonding_curve: accounts[4],
            associated_user: accounts[5],
            user: accounts[6],
            creator_vault: accounts[8],
            min_sol_output,
            amount,
            is_buy: false,
            ..Default::default()
        }))
    }

    fn parse_log_inner_instruction(
        _data: &[u8],
        metadata: EventMetadata,
        log_messages: &Option<Vec<String>>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if let Some(logs) = log_messages {
            let re = Regex::new(r"sol_amount: (\d+), token_amount: (\d+)").unwrap();
            for log in logs {
                if let Some(captures) = re.captures(log) {
                    let sol_amount = captures.get(1).unwrap().as_str().parse::<u64>().unwrap();
                    let token_amount = captures.get(2).unwrap().as_str().parse::<u64>().unwrap();

                    let mut event = PumpFunTradeEvent {
                        metadata: metadata.clone(),
                        sol_amount,
                        token_amount,
                        ..Default::default()
                    };

                    let mut metadata = metadata.clone();
                    metadata.set_id(format!(
                        "{}-{}-{}",
                        metadata.signature,
                        metadata.index,
                        "trade_log"
                    ));
                    event.metadata = metadata;

                    return Some(Box::new(event));
                }
            }
        }
        None
    }

    fn parse_log_instruction(
        _data: &[u8],
        _accounts: &[Pubkey],
        metadata: EventMetadata,
        log_messages: &Option<Vec<String>>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if let Some(logs) = log_messages {
            let re = Regex::new(r"sol_amount: (\d+), token_amount: (\d+)").unwrap();
            for log in logs {
                if let Some(captures) = re.captures(log) {
                    let sol_amount = captures.get(1).unwrap().as_str().parse::<u64>().unwrap();
                    let token_amount = captures.get(2).unwrap().as_str().parse::<u64>().unwrap();

                    let mut event = PumpFunTradeEvent {
                        metadata: metadata.clone(),
                        sol_amount,
                        token_amount,
                        ..Default::default()
                    };

                    let mut metadata = metadata.clone();
                    metadata.set_id(format!(
                        "{}-{}-{}",
                        metadata.signature,
                        metadata.index,
                        "trade_log"
                    ));
                    event.metadata = metadata;

                    return Some(Box::new(event));
                }
            }
        }
        None
    }
}

#[async_trait::async_trait]
impl EventParser for PumpFunEventParser {
    fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<Timestamp>,
        index: String,
        log_messages: &Option<Vec<String>>,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        self.inner.parse_events_from_inner_instruction(
            inner_instruction,
            signature,
            slot,
            block_time,
            index,
            log_messages,
        )
    }

    fn parse_events_from_instruction(
        &self,
        instruction: &CompiledInstruction,
        accounts: &[Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<Timestamp>,
        index: String,
        log_messages: &Option<Vec<String>>,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        self.inner.parse_events_from_instruction(
            instruction,
            accounts,
            signature,
            slot,
            block_time,
            index,
            log_messages,
        )
    }

    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.inner.should_handle(program_id)
    }

    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.inner.supported_program_ids()
    }
}
