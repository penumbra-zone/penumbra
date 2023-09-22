use penumbra_chain::parameters::ChainParameters;
use penumbra_dao::parameters::DaoParameters;
use penumbra_governance::parameters::GovernanceParameters;
use penumbra_ibc::parameters::IbcParameters;
use penumbra_stake::parameters::StakeParameters;

pub struct AppParameters {
    pub stake_parameters: StakeParameters,
    pub ibc_parameters: IbcParameters,
    pub governance_parameters: GovernanceParameters,
    pub chain_parameters: ChainParameters,
    pub dao_parameters: DaoParameters,
}
