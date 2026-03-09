use std::fmt;

use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use snafu::ensure;

use crate::domain::error::{DomainResult, QuotaExceededSnafu, ValidationSnafu};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuotaState {
    used: Byte,
    limit: Byte,
}

impl QuotaState {
    pub fn new(used: Byte, limit: Byte, max_limit: Byte) -> DomainResult<Self> {
        ensure!(
            limit.as_u64() <= max_limit.as_u64(),
            ValidationSnafu {
                message: "The quota limit exceeds the maximum allowed limit"
            }
        );
        ensure!(
            used.as_u64() <= limit.as_u64(),
            QuotaExceededSnafu {
                required: used,
                available: limit,
            }
        );

        Ok(Self { used, limit })
    }

    pub fn new_unchecked(used: Byte, limit: Byte) -> Self {
        Self { used, limit }
    }

    pub fn remaining(&self) -> Byte {
        if self.used.as_u64() > self.limit.as_u64() {
            Byte::from_u64(0)
        } else {
            Byte::from_u64(self.limit.as_u64() - self.used.as_u64())
        }
    }

    pub fn reserve_quota(&mut self, additional: Byte) -> DomainResult<()> {
        ensure!(
            self.used.as_u64() + additional.as_u64() <= self.limit.as_u64(),
            QuotaExceededSnafu {
                required: additional,
                available: self.limit,
            }
        );

        self.used = Byte::from_u64(self.used.as_u64() + additional.as_u64());
        Ok(())
    }

    pub fn release_quota(&mut self, released: Byte) {
        self.used = Byte::from_u64(self.used.as_u64() - released.as_u64());
    }

    pub fn used(&self) -> Byte {
        self.used
    }

    pub fn limit(&self) -> Byte {
        self.limit
    }
}

impl fmt::Display for QuotaState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} / {}", self.used, self.limit)
    }
}
