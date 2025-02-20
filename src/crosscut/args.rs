use pallas::network::miniprotocols::{Point, MAINNET_MAGIC, TESTNET_MAGIC};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref, str::FromStr};

use crate::Error;

/// A serialization-friendly chain Point struct using a hex-encoded hash
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PointArg {
    Origin,
    Specific(u64, String),
}

impl TryInto<Point> for PointArg {
    type Error = crate::Error;

    fn try_into(self) -> Result<Point, Self::Error> {
        match self {
            PointArg::Origin => Ok(Point::Origin),
            PointArg::Specific(slot, hash_hex) => {
                let hash = hex::decode(&hash_hex)
                    .map_err(|_| Self::Error::message("can't decode point hash hex value"))?;

                Ok(Point::Specific(slot, hash))
            }
        }
    }
}

impl From<Point> for PointArg {
    fn from(other: Point) -> Self {
        match other {
            Point::Origin => PointArg::Origin,
            Point::Specific(slot, hash) => PointArg::Specific(slot, hex::encode(hash)),
        }
    }
}

impl FromStr for PointArg {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            x if s.contains(',') => {
                let mut parts: Vec<_> = x.split(',').collect();
                let slot = parts
                    .remove(0)
                    .parse()
                    .map_err(|_| Self::Err::message("can't parse slot number"))?;

                let hash = parts.remove(0).to_owned();
                Ok(PointArg::Specific(slot, hash))
            }
            "origin" => Ok(PointArg::Origin),
            _ => Err(Self::Err::message(
                "Can't parse chain point value, expecting `slot,hex-hash` format",
            )),
        }
    }
}

impl ToString for PointArg {
    fn to_string(&self) -> String {
        match self {
            PointArg::Origin => "origin".to_string(),
            PointArg::Specific(slot, hash) => format!("{},{}", slot, hash),
        }
    }
}

pub type Cursor = Option<PointArg>;

#[derive(Debug, Deserialize, Clone)]
pub struct MagicArg(pub u64);

impl Deref for MagicArg {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MagicArg {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let m = match s {
            "testnet" => MagicArg(TESTNET_MAGIC),
            "mainnet" => MagicArg(MAINNET_MAGIC),
            _ => MagicArg(u64::from_str(s).map_err(|_| "can't parse magic value")?),
        };

        Ok(m)
    }
}

impl Default for MagicArg {
    fn default() -> Self {
        Self(MAINNET_MAGIC)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum IntersectConfig {
    Tip,
    Origin,
    Point(PointArg),
    Fallbacks(Vec<PointArg>),
}

/// Well-known information about the blockhain network
///
/// Some of the logic in Scrolls depends on particular characteristic of the
/// network that it's consuming from. For example: time calculation and bech32
/// encoding. This struct groups all of these blockchain network specific
/// values.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainWellKnownInfo {
    pub magic: u64,
    pub byron_epoch_length: u32,
    pub byron_slot_length: u32,
    pub byron_known_slot: u64,
    pub byron_known_hash: String,
    pub byron_known_time: u64,
    pub shelley_epoch_length: u32,
    pub shelley_slot_length: u32,
    pub shelley_known_slot: u64,
    pub shelley_known_hash: String,
    pub shelley_known_time: u64,
    pub address_hrp: String,
    pub adahandle_policy: String,
}

impl ChainWellKnownInfo {
    /// Hardcoded values for mainnet
    pub fn mainnet() -> Self {
        ChainWellKnownInfo {
            magic: MAINNET_MAGIC,
            byron_epoch_length: 432000,
            byron_slot_length: 20,
            byron_known_slot: 0,
            byron_known_time: 1506203091,
            byron_known_hash: "f0f7892b5c333cffc4b3c4344de48af4cc63f55e44936196f365a9ef2244134f"
                .to_string(),
            shelley_epoch_length: 432000,
            shelley_slot_length: 1,
            shelley_known_slot: 4492800,
            shelley_known_hash: "aa83acbf5904c0edfe4d79b3689d3d00fcfc553cf360fd2229b98d464c28e9de"
                .to_string(),
            shelley_known_time: 1596059091,
            address_hrp: "addr".to_string(),
            adahandle_policy: "f0ff48bbb7bbe9d59a40f1ce90e9e9d0ff5002ec48f232b49ca0fb9a"
                .to_string(),
        }
    }

    /// Hardcoded values for testnet
    pub fn testnet() -> Self {
        ChainWellKnownInfo {
            magic: TESTNET_MAGIC,
            byron_epoch_length: 432000,
            byron_slot_length: 20,
            byron_known_slot: 0,
            byron_known_time: 1564010416,
            byron_known_hash: "8f8602837f7c6f8b8867dd1cbc1842cf51a27eaed2c70ef48325d00f8efb320f"
                .to_string(),
            shelley_epoch_length: 432000,
            shelley_slot_length: 1,
            shelley_known_slot: 1598400,
            shelley_known_hash: "02b1c561715da9e540411123a6135ee319b02f60b9a11a603d3305556c04329f"
                .to_string(),
            shelley_known_time: 1595967616,
            address_hrp: "addr_test".to_string(),
            adahandle_policy: "8d18d786e92776c824607fd8e193ec535c79dc61ea2405ddf3b09fe3"
                .to_string(),
        }
    }

    /// Uses the value of the magic to return either mainnet or testnet
    /// hardcoded values.
    pub fn try_from_magic(magic: u64) -> Result<ChainWellKnownInfo, Error> {
        match magic {
            MAINNET_MAGIC => Ok(Self::mainnet()),
            TESTNET_MAGIC => Ok(Self::testnet()),
            _ => Err(Error::ConfigError(
                "can't infer well-known chain infro from specified magic".into(),
            )),
        }
    }
}

impl Default for ChainWellKnownInfo {
    fn default() -> Self {
        Self::mainnet()
    }
}
