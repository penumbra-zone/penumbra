use penumbra_proto::core::component::fee::v1 as pb;

use crate::{Fee, Gas};

fn fee_payment(fee: Fee, base_fee: Fee, gas_used: Gas) -> pb::EventFeePayment {
    pb::EventFeePayment {
        fee: Some(fee.into()),
        base_fee: Some(base_fee.into()),
        gas_used: Some(gas_used.into()),
    }
}
