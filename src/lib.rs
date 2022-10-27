#![feature(const_mut_refs)]

use serde::Deserialize;
use std::collections::BTreeMap;
use strum::Display;

pub type FnPtr<T> = dyn Sync + Fn(&mut T);
pub type FnByte = fn(&[u8]) -> Option<Vec<u8>>;
pub type FnDesc = fn() -> (&'static str, &'static str);

#[derive(Default, Deserialize, Display, PartialEq, Eq)]
pub enum Status {
    Enabled,
    #[default]
    Disabled,
}

impl Status {
    pub fn enabled() -> Self {
        Status::Enabled
    }

    pub fn disabled() -> Self {
        Status::Disabled
    }
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub block_height: u64,
    pub issue: Option<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            block_height: 1,
            issue: None,
        }
    }
}

#[derive(Clone)]
pub struct InnerMig<'a, T> {
    i: &'a FnPtr<T>,
    u: &'a FnPtr<T>,
    r: FnByte,
    pub d: FnDesc,
}

pub struct Mig<'a, T> {
    pub mig: InnerMig<'a, T>,
    pub metadata: Metadata,
    pub status: Status,
}

impl<'a, T> Mig<'a, T> {
    pub const fn new(mig: InnerMig<'a, T>, metadata: Metadata, status: Status) -> Self {
        Self {
            mig,
            metadata,
            status,
        }
    }

    /// This function gets executed when the storage block height == the migration block height
    pub fn init(&self, s: &mut T, h: u64) {
        if self.status == Status::Enabled && self.metadata().block_height == h {
            (self.mig.i)(s);
        }
    }

    /// This function gets executed when the storage block height >= the migration block height
    pub fn update(&self, s: &mut T, h: u64) {
        if self.status == Status::Enabled && self.metadata().block_height >= h {
            (self.mig.u)(s);
        }
    }

    pub fn run<'b>(&'b self, b: &'b [u8], h: u64) -> Option<Vec<u8>> {
        if self.status == Status::Enabled && self.metadata().block_height == h {
            return (self.mig.r)(b);
        }
        None
    }

    pub fn desc(&self) -> (&'static str, &'static str) {
        (self.mig.d)()
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn disable(&mut self) {
        self.status = Status::Disabled
    }

    pub fn enable(&mut self) {
        self.status = Status::Enabled
    }

    pub fn is_enabled(&self) -> bool {
        self.status == Status::Enabled
    }
}

fn noop_b(_b: &[u8]) -> Option<Vec<u8>> {
    None
}

impl<'a, T> InnerMig<'a, T> {
    pub const fn new(i: &'a FnPtr<T>, u: &'a FnPtr<T>, r: FnByte, d: FnDesc) -> Self {
        Self { i, u, r, d }
    }

    pub const fn new_run(r: FnByte, d: FnDesc) -> Self {
        Self {
            i: &|_| {},
            u: &|_| {},
            r,
            d,
        }
    }

    pub const fn new_init(i: &'a FnPtr<T>, d: FnDesc) -> Self {
        Self {
            i,
            u: &|_| {},
            r: noop_b,
            d,
        }
    }

    pub const fn new_update(u: &'a FnPtr<T>, d: FnDesc) -> Self {
        Self {
            i: &|_| {},
            u,
            r: noop_b,
            d,
        }
    }

    pub const fn new_init_update(i: &'a FnPtr<T>, u: &'a FnPtr<T>, d: FnDesc) -> Self {
        Self { i, u, r: noop_b, d }
    }

    /// This function gets executed when the storage block height == the migration block height
    pub fn init(&self, s: &mut T) {
        (self.i)(s);
    }

    /// This function gets executed when the storage block height >= the migration block height
    pub fn update(&self, s: &mut T) {
        (self.u)(s);
    }

    pub fn desc(&self) -> (&'static str, &'static str) {
        (self.d)()
    }

    pub fn run<'b>(&'b self, b: &'b [u8]) -> Option<Vec<u8>> {
        (self.r)(b)
    }
}

#[derive(Deserialize)]
struct IO<'a> {
    r#type: &'a str,

    #[serde(flatten)]
    metadata: Metadata,
}

pub fn load_migrations<'de: 'a, 'a, T: Clone>(
    registry: &[InnerMig<'a, T>],
    data: &'a str,
) -> Result<BTreeMap<&'a str, Mig<'a, T>>, String> {
    // TODO: Do not hardcode the deserializer
    let config: Vec<IO> = serde_json::from_str(data).unwrap();

    // Build a BTreeMap from the linear registry
    let registry = registry
        .iter()
        .map(|m| ((m.d)().0, m))
        .collect::<BTreeMap<&'a str, &InnerMig<'a, T>>>();

    Ok(config
        .into_iter()
        .map(|io| {
            let (&k, &v) = registry
                .get_key_value(io.r#type)
                .ok_or_else(|| format!("Unsupported migration type {}", io.r#type))?;
            Ok((k, Mig::new(v.clone(), io.metadata, Status::Enabled)))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?
        .into_iter()
        .collect())
}

pub fn load_enable_all_migrations<'a, T: Clone>(
    registry: &[InnerMig<'a, T>],
) -> BTreeMap<&'a str, Mig<'a, T>> {
    registry
        .iter()
        .map(|m| {
            (
                (m.d)().0,
                Mig::new(m.clone(), Metadata::default(), Status::Enabled),
            )
        })
        .collect()
}
