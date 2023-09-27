use penumbra_chain::parameters::ChainParameters;
use penumbra_dao::parameters::DaoParameters;
use penumbra_governance::params::GovernanceParameters;
use penumbra_ibc::params::IBCParameters;
use penumbra_stake::params::StakeParameters;

pub mod change;

pub struct AppParameters {
    pub stake_params: StakeParameters,
    pub ibc_params: IBCParameters,
    pub governance_params: GovernanceParameters,
    pub chain_params: ChainParameters,
    pub dao_params: DaoParameters,
}
