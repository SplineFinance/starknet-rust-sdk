use crate::math::uint::U256;
use crate::quoting::base_pool::{
    BasePool, BasePoolResources, BasePoolState, BasePoolQuoteError
};
use crate::quoting::types::BlockTimestamp;
use crate::quoting::types::{NodeKey, Pool, Quote, QuoteParams, Tick};
use alloc::vec;
use alloc::vec::Vec;
use core::ops::Add;

#[derive(Clone, Copy)]
pub struct SplinePoolState {
    pub base_pool_state: BasePoolState,
    pub liquidity_factor: u128,
    pub total_shares: U256,
}

#[derive(Default, Clone, Copy)]
pub struct SplinePoolResources {
    pub base_pool_resources: BasePoolResources,
    pub fee_compounds: u32,
}

impl Add for SplinePoolResources {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            base_pool_resources: self.base_pool_resources + rhs.base_pool_resources,
            fee_compounds: self.fee_compounds + rhs.fee_compounds,
        }
    }
}

pub struct SplinePool {
    base_pool: BasePool,
    liquidity_factor: u128,
    total_shares: U256,
}

impl SplinePool {
    pub fn new(key: NodeKey, state: SplinePoolState, sorted_ticks: Vec<Tick>) -> Self {
        SplinePool {
            base_pool: BasePool::new(
                key,
                state.base_pool_state,
                sorted_ticks,
            ),
            liquidity_factor: state.liquidity_factor,
            total_shares: state.liquidity_factor.into(),
        }
    }
}

impl Pool for SplinePool {
    type Resources = SplinePoolResources;
    type State = SplinePoolState;
    type QuoteError = BasePoolQuoteError;
    type Meta = BlockTimestamp;

    fn get_key(&self) -> &NodeKey {
        self.base_pool.get_key()
    }

    fn get_state(&self) -> Self::State {
        SplinePoolState {
            base_pool_state: self.base_pool.get_state(),
            liquidity_factor: self.liquidity_factor,
            total_shares: self.liquidity_factor.into(),
        }
    }

    fn quote(
        &self,
        params: QuoteParams<Self::State, Self::Meta>,
    ) -> Result<Quote<Self::Resources, Self::State>, Self::QuoteError> {
        let fee_compounds = 0;
        
        let liquidity_factor = params.override_state.map_or(self.liquidity_factor, |os| os.liquidity_factor);
        
        let result = self.base_pool.quote(QuoteParams {
            sqrt_ratio_limit: params.sqrt_ratio_limit,
            override_state: params.override_state.map(|s| s.base_pool_state),
            token_amount: params.token_amount,
            meta: (),
        })?;
        
        Ok(Quote {
            calculated_amount: result.calculated_amount,
            consumed_amount: result.consumed_amount,
            execution_resources: SplinePoolResources {
                base_pool_resources: result.execution_resources,
                fee_compounds,
            },
            fees_paid: result.fees_paid,
            is_price_increasing: result.is_price_increasing,
            state_after: SplinePoolState {
                base_pool_state: result.state_after,
                liquidity_factor,
                total_shares: self.total_shares,
            },
        })
    }

    fn has_liquidity(&self) -> bool {
        self.base_pool.has_liquidity()
    }
    
    fn max_tick_with_liquidity(&self) -> Option<i32> {
        self.base_pool.max_tick_with_liquidity()
    }

    fn min_tick_with_liquidity(&self) -> Option<i32> {
        self.base_pool.min_tick_with_liquidity()
    }
}

