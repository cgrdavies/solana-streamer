use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::streaming::event_parser::{
    common::{read_u64_le, EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::raydium_cpmm::{discriminators, RaydiumCpmmSwapEvent},
};

/// Raydium CPMM程序ID
pub const RAYDIUM_CPMM_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");

/// Raydium CPMM事件解析器
pub struct RaydiumCpmmEventParser {
    inner: GenericEventParser,
}

impl RaydiumCpmmEventParser {
    pub fn new() -> Self {
        // 配置所有事件类型
        let configs = vec![
            GenericEventParseConfig {
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::SWAP_BASE_IN,
                event_type: EventType::RaydiumCpmmSwapBaseInput,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_swap_base_input_instruction,
            },
            GenericEventParseConfig {
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::SWAP_BASE_OUT,
                event_type: EventType::RaydiumCpmmSwapBaseOutput,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_swap_base_output_instruction,
            },
        ];

        let inner =
            GenericEventParser::new(RAYDIUM_CPMM_PROGRAM_ID, ProtocolType::RaydiumCpmm, configs);

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

    /// 解析买入指令事件
    fn parse_swap_base_input_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
        _log_messages: &Option<Vec<String>>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 13 {
            return None;
        }

        let amount_in = read_u64_le(data, 0)?;
        let minimum_amount_out = read_u64_le(data, 8)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[3], accounts[10], accounts[11]
        ));

        Some(Box::new(RaydiumCpmmSwapEvent {
            metadata,
            amount_in,
            minimum_amount_out,
            payer: accounts[0],
            authority: accounts[1],
            amm_config: accounts[2],
            pool_state: accounts[3],
            input_token_account: accounts[4],
            output_token_account: accounts[5],
            input_vault: accounts[6],
            output_vault: accounts[7],
            input_token_mint: accounts[10],
            output_token_mint: accounts[11],
            observation_state: accounts[12],
            ..Default::default()
        }))
    }

    fn parse_swap_base_output_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
        _log_messages: &Option<Vec<String>>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 13 {
            return None;
        }

        let max_amount_in = read_u64_le(data, 0)?;
        let amount_out = read_u64_le(data, 8)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[3], accounts[10], accounts[11]
        ));

        Some(Box::new(RaydiumCpmmSwapEvent {
            metadata,
            max_amount_in,
            amount_out,
            payer: accounts[0],
            authority: accounts[1],
            amm_config: accounts[2],
            pool_state: accounts[3],
            input_token_account: accounts[4],
            output_token_account: accounts[5],
            input_vault: accounts[6],
            output_vault: accounts[7],
            input_token_mint: accounts[10],
            output_token_mint: accounts[11],
            observation_state: accounts[12],
            ..Default::default()
        }))
    }
}

#[async_trait::async_trait]
impl EventParser for RaydiumCpmmEventParser {
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
