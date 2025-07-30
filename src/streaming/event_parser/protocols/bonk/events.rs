use crate::streaming::event_parser::protocols::bonk::types::{
    CurveParams, MintParams, PoolStatus, TradeDirection, VestingParams,
};
use crate::streaming::event_parser::common::EventMetadata;
use crate::impl_unified_event;
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// 买入事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BonkTradeEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool_state: Pubkey,
    pub total_base_sell: u64,
    pub virtual_base: u64,
    pub virtual_quote: u64,
    pub real_base_before: u64,
    pub real_quote_before: u64,
    pub real_base_after: u64,
    pub real_quote_after: u64,
    pub amount_in: u64,
    pub amount_out: u64,
    pub protocol_fee: u64,
    pub platform_fee: u64,
    pub share_fee: u64,
    pub trade_direction: TradeDirection,
    pub pool_status: PoolStatus,
    #[borsh(skip)]
    pub minimum_amount_out: u64,
    #[borsh(skip)]
    pub maximum_amount_in: u64,
    #[borsh(skip)]
    pub share_fee_rate: u64,
    #[borsh(skip)]
    pub payer: Pubkey,
    #[borsh(skip)]
    pub user_base_token: Pubkey,
    #[borsh(skip)]
    pub user_quote_token: Pubkey,
    #[borsh(skip)]
    pub base_vault: Pubkey,
    #[borsh(skip)]
    pub quote_vault: Pubkey,
    #[borsh(skip)]
    pub base_token_mint: Pubkey,
    #[borsh(skip)]
    pub quote_token_mint: Pubkey,
    #[borsh(skip)]
    pub is_dev_create_token_trade: bool,
    #[borsh(skip)]
    pub is_bot: bool,
}

// 使用宏生成UnifiedEvent实现，指定需要合并的字段
impl_unified_event!(
    BonkTradeEvent,
    pool_state,
    total_base_sell,
    virtual_base,
    virtual_quote,
    real_base_before,
    real_quote_before,
    real_base_after,
    real_quote_after,
    amount_in,
    amount_out,
    protocol_fee,
    platform_fee,
    share_fee,
    trade_direction,
    pool_status
);

/// 创建池事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BonkPoolCreateEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool_state: Pubkey,
    pub creator: Pubkey,
    pub config: Pubkey,
    pub base_mint_param: MintParams,
    pub curve_param: CurveParams,
    pub vesting_param: VestingParams,
    #[borsh(skip)]
    pub payer: Pubkey,
    #[borsh(skip)]
    pub base_mint: Pubkey,
    #[borsh(skip)]
    pub quote_mint: Pubkey,
    #[borsh(skip)]
    pub base_vault: Pubkey,
    #[borsh(skip)]
    pub quote_vault: Pubkey,
    #[borsh(skip)]
    pub global_config: Pubkey,
    #[borsh(skip)]
    pub platform_config: Pubkey,
}

// 使用宏生成UnifiedEvent实现，指定需要合并的字段
impl_unified_event!(
    BonkPoolCreateEvent,
    pool_state,
    creator,
    config,
    base_mint_param,
    curve_param,
    vesting_param
);

/// 事件鉴别器常量
pub mod discriminators {
    // 事件鉴别器
    pub const TRADE_EVENT: &str = "0xe445a52e51cb9a1dbddb7fd34ee661ee";
    pub const POOL_CREATE_EVENT: &str = "0xe445a52e51cb9a1d97d7e20976a173ae";

    // 指令鉴别器
    pub const BUY_EXACT_IN: &[u8] = &[250, 234, 13, 123, 213, 156, 19, 236];
    pub const BUY_EXACT_OUT: &[u8] = &[24, 211, 116, 40, 105, 3, 153, 56];
    pub const SELL_EXACT_IN: &[u8] = &[149, 39, 222, 155, 211, 124, 152, 26];
    pub const SELL_EXACT_OUT: &[u8] = &[95, 200, 71, 34, 8, 9, 11, 166];
    pub const INITIALIZE: &[u8] = &[175, 175, 109, 31, 13, 152, 155, 237];
}
