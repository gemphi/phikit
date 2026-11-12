// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/work_balancer.h` and
//! `work_balancer.cpp` (`WorkBalanceInfo`, `work_balance_do_initial`,
//! `work_balance_do_rebalance`): how a frame is shared between devices, so
//! each spends the same time on its share.

/// `WorkBalanceInfo`: one device's share of the work.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkBalanceInfo {
    /// Seconds spent path tracing since the last rebalance.
    pub time_spent: f64,
    /// How full the device's path states were, on average.
    pub occupancy: f32,
    /// The device's fraction of the frame. The weights add up to one.
    pub weight: f64,
}

impl Default for WorkBalanceInfo {
    fn default() -> Self {
        Self { time_spent: 0.0, occupancy: 1.0, weight: 1.0 }
    }
}

impl WorkBalanceInfo {
    /// `work_balance_do_initial`: nothing was timed yet, so equal shares.
    pub fn share_evenly(list: &mut [WorkBalanceInfo]) {
        let weight = 1.0 / list.len() as f64;
        list.iter_mut().for_each(|info| info.weight = weight);
    }

    /// `work_balance_do_rebalance`: move the shares toward equal times.
    /// `false`, with nothing changed, when the times are within 2% already.
    ///
    /// A device 10% faster than another is not handed all of the
    /// difference: each moves a fraction of the way to the average, so the
    /// times meet after the weights change.
    pub fn rebalance(list: &mut [WorkBalanceInfo]) -> bool {
        let count = list.len() as f64;
        let time_average = list.iter().map(|info| info.time_spent).sum::<f64>() / count;
        let target = |info: &WorkBalanceInfo| info.time_spent + (time_average - info.time_spent) / count;
        let has_big_difference = list.iter().any(|info| (1.0 - target(info) / time_average).abs() > 0.02);
        if !has_big_difference {
            return false;
        }
        let new_weight: Vec<f64> = list.iter().map(|info| info.weight * target(info) / info.time_spent).collect();
        let total_weight: f64 = new_weight.iter().sum();
        for (info, weight) in list.iter_mut().zip(new_weight) {
            info.weight = weight / total_weight;
            info.time_spent = 0.0;
        }
        true
    }
}

#[cfg(test)]
mod test;
