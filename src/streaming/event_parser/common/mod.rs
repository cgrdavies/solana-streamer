pub mod types;
pub mod utils;

/// 自动生成UnifiedEvent trait实现的宏
#[macro_export]
macro_rules! impl_unified_event {
    // 带有自定义ID表达式的版本
    ($struct_name:ident, $($field:ident),*) => {
        impl $crate::streaming::event_parser::core::traits::UnifiedEvent for $struct_name {
            fn id(&self) -> &str {
                &self.metadata.id
            }

            fn event_type(&self) -> $crate::streaming::event_parser::common::types::EventType {
                self.metadata.event_type.clone()
            }

            fn signature(&self) -> &str {
                &self.metadata.signature
            }

            fn slot(&self) -> u64 {
                self.metadata.slot
            }

            fn program_received_time_ms(&self) -> i64 {
                self.metadata.program_received_time_ms
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn clone_boxed(&self) -> Box<dyn $crate::streaming::event_parser::core::traits::UnifiedEvent> {
                Box::new(self.clone())
            }

            fn merge(&mut self, other: Box<dyn $crate::streaming::event_parser::core::traits::UnifiedEvent>) {
                if let Some(_e) = other.as_any().downcast_ref::<$struct_name>() {
                    $(
                        self.$field = _e.$field.clone();
                    )*
                }
            }

            fn set_transfer_datas(&mut self, transfer_datas: Vec<$crate::streaming::event_parser::common::types::TransferData>) {
                self.metadata.transfer_datas = transfer_datas;
            }

            fn index(&self) -> String {
                self.metadata.index.clone()
            }
        }
    };
}

pub use types::*;
pub use utils::*;
