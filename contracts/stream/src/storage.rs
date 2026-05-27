use soroban_sdk::{Env, Address};
use crate::types::{DataKey, Stream, StreamStatus};

pub fn save_stream(env: &Env, stream: &Stream) {
    env.storage().persistent().set(&DataKey::Stream(stream.id), stream);
}

pub fn load_stream(env: &Env, id: u64) -> Option<Stream> {
    env.storage().persistent().get(&DataKey::Stream(id))
}

pub fn next_id(env: &Env) -> u64 {
    let count: u64 = env.storage().instance().get(&DataKey::StreamCount).unwrap_or(0);
    let next = count + 1;
    env.storage().instance().set(&DataKey::StreamCount, &next);
    next
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).expect("admin not set")
}

/// Tokens earned by employee up to `now` that have not yet been withdrawn.
pub fn claimable_amount(stream: &Stream, now: u64) -> i128 {
    if stream.status == StreamStatus::Cancelled || stream.status == StreamStatus::Exhausted {
        return 0;
    }

    // Vesting streams: linear unlock between `start_time` and `vest_end`.
    if stream.is_vesting {
        if stream.vest_end == 0 || stream.vest_total <= 0 {
            return 0;
        }
        let unlocked = if now >= stream.vest_end {
            stream.vest_total
        } else if now <= stream.start_time {
            0
        } else {
            // unlocked = vest_total * (now - start_time) / (vest_end - start_time)
            let numer = (now.saturating_sub(stream.start_time)) as i128;
            let denom = (stream.vest_end.saturating_sub(stream.start_time)) as i128;
            (stream.vest_total * numer / denom).max(0)
        };
        let claimable = unlocked - stream.withdrawn;
        return claimable.max(0);
    }

    // Streaming (per-second) behaviour (existing logic)
    let effective_end = if stream.stop_time > 0 {
        now.min(stream.stop_time)
    } else {
        now
    };
    let elapsed = effective_end.saturating_sub(stream.last_withdraw_time) as i128;
    let earned = elapsed * stream.rate_per_second;
    let remaining = stream.deposit - stream.withdrawn;
    earned.min(remaining).max(0)
}
