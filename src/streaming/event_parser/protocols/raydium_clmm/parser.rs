use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::streaming::event_parser::{
    common::{read_u128_le, read_u64_le, read_u8_le, EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::raydium_clmm::{discriminators, RaydiumClmmSwapEvent, RaydiumClmmSwapV2Event},
};

/// Raydium CLMM程序ID
pub const RAYDIUM_CLMM_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");

/// Raydium CLMM事件解析器
pub struct RaydiumClmmEventParser {
    inner: GenericEventParser,
}

impl RaydiumClmmEventParser {
    pub fn new() -> Self {
        // 配置所有事件类型
        let configs = vec![
            GenericEventParseConfig {
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::SWAP,
                event_type: EventType::RaydiumClmmSwap,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_swap_instruction,
            },
            GenericEventParseConfig {
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::SWAP_V2,
                event_type: EventType::RaydiumClmmSwapV2,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_swap_v2_instruction,
            },
        ];

        let inner =
            GenericEventParser::new(RAYDIUM_CLMM_PROGRAM_ID, ProtocolType::RaydiumClmm, configs);

        Self { inner }
    }

    /// 解析交易事件
    fn parse_trade_inner_instruction(
        _data: &[u8],
        _metadata: EventMetadata,
        _log_messages: &Option<Vec<String>>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        None
    }

    /// 解析交易指令事件
    fn parse_swap_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
        _log_messages: &Option<Vec<String>>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 10 {
            return None;
        }

        let amount = read_u64_le(data, 0)?;
        let other_amount_threshold = read_u64_le(data, 8)?;
        let sqrt_price_limit_x64 = read_u128_le(data, 16)?;
        let is_base_input = read_u8_le(data, 32)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[2], accounts[3], accounts[4]
        ));

        Some(Box::new(RaydiumClmmSwapEvent {
            metadata,
            amount,
            other_amount_threshold,
            sqrt_price_limit_x64,
            is_base_input: is_base_input == 1,
            payer: accounts[0],
            amm_config: accounts[1],
            pool_state: accounts[2],
            input_token_account: accounts[3],
            output_token_account: accounts[4],
            input_vault: accounts[5],
            output_vault: accounts[6],
            observation_state: accounts[7],
            token_program: accounts[8],
            tick_array: accounts[9],
            remaining_accounts: accounts[10..].to_vec(),
            ..Default::default()
        }))
    }

    fn parse_swap_v2_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
        _log_messages: &Option<Vec<String>>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 13 {
            return None;
        }

        let amount = read_u64_le(data, 0)?;
        let other_amount_threshold = read_u64_le(data, 8)?;
        let sqrt_price_limit_x64 = read_u128_le(data, 16)?;
        let is_base_input = read_u8_le(data, 32)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[2], accounts[3], accounts[4]
        ));

        Some(Box::new(RaydiumClmmSwapV2Event {
            metadata,
            amount,
            other_amount_threshold,
            sqrt_price_limit_x64,
            is_base_input: is_base_input == 1,
            payer: accounts[0],
            amm_config: accounts[1],
            pool_state: accounts[2],
            input_token_account: accounts[3],
            output_token_account: accounts[4],
            input_vault: accounts[5],
            output_vault: accounts[6],
            observation_state: accounts[7],
            token_program: accounts[8],
            token_program2022: accounts[9],
            memo_program: accounts[10],
            input_vault_mint: accounts[11],
            output_vault_mint: accounts[12],
            remaining_accounts: accounts[13..].to_vec(),
            ..Default::default()
        }))
    }
}

#[async_trait::async_trait]
impl EventParser for RaydiumClmmEventParser {
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
