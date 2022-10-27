#![feature(const_mut_refs)]

use serde::Deserialize;
use std::collections::BTreeMap;
use strum::Display;
use tracing::trace;

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

// TODO: Add the serde additional field dictionary
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
pub enum MigrationType<'a, T> {
    Regular(RegularMigration<'a, T>),
    Hotfix(HotfixMigration),
}

#[derive(Clone)]
pub struct RegularMigration<'a, T> {
    initialize_fn: &'a FnPtr<T>,
    update_fn: &'a FnPtr<T>,
}

#[derive(Clone)]
pub struct HotfixMigration {
    hotfix_fn: FnByte,
}

// TODO: Split this in two (enums)
// TODO: Use full words, i.e., "Migration" and not "Mig"
#[derive(Clone)]
pub struct InnerMigration<'a, T> {
    r#type: MigrationType<'a, T>,
    name: &'a str,
    description: &'a str,
}

pub struct Migration<'a, T> {
    pub migration: InnerMigration<'a, T>,
    pub metadata: Metadata,
    pub status: Status,
}

impl<'a, T> Migration<'a, T> {
    pub const fn new(migration: InnerMigration<'a, T>, metadata: Metadata, status: Status) -> Self {
        Self {
            migration,
            metadata,
            status,
        }
    }

    /// This function gets executed when the storage block height == the migration block height
    pub fn initialize(&self, storage: &mut T, h: u64) {
        if self.status == Status::Enabled && self.metadata().block_height == h {
            trace!("Trying to initialize migration - {}", self.name());
            self.migration.initialize(storage);
        }
    }

    /// This function gets executed when the storage block height >= the migration block height
    pub fn update(&self, storage: &mut T, h: u64) {
        if self.status == Status::Enabled && self.metadata().block_height >= h {
            trace!("Trying to update migration - {}", self.name());
            self.migration.update(storage);
        }
    }

    /// This function gets executed when the storage block height == the migration block height
    pub fn hotfix<'b>(&'b self, b: &'b [u8], h: u64) -> Option<Vec<u8>> {
        if self.status == Status::Enabled && self.metadata().block_height == h {
            trace!("Trying to execute hotfix - {}", self.name());
            return self.migration.hotfix(b);
        }
        None
    }

    pub fn name(&self) -> &'a str {
        self.migration.name()
    }

    pub fn description(&self) -> &'a str {
        self.migration.description()
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

impl<'a, T> InnerMigration<'a, T> {
    pub const fn new_hotfix(hotfix_fn: FnByte, name: &'a str, description: &'a str) -> Self {
        Self {
            r#type: MigrationType::Hotfix(HotfixMigration { hotfix_fn }),
            name,
            description,
        }
    }

    pub const fn new_initialize_update(
        initialize_fn: &'a FnPtr<T>,
        update_fn: &'a FnPtr<T>,
        name: &'a str,
        description: &'a str,
    ) -> Self {
        Self {
            r#type: MigrationType::Regular(RegularMigration {
                initialize_fn,
                update_fn,
            }),
            name,
            description,
        }
    }

    pub const fn new_initialize(
        initialize_fn: &'a FnPtr<T>,
        name: &'a str,
        description: &'a str,
    ) -> Self {
        Self {
            r#type: MigrationType::Regular(RegularMigration {
                initialize_fn,
                update_fn: &|_| {},
            }),
            name,
            description,
        }
    }

    pub const fn new_update(update_fn: &'a FnPtr<T>, name: &'a str, description: &'a str) -> Self {
        Self {
            r#type: MigrationType::Regular(RegularMigration {
                initialize_fn: &|_| {},
                update_fn,
            }),
            name,
            description,
        }
    }

    pub const fn name(&self) -> &'a str {
        self.name
    }

    pub const fn description(&self) -> &'a str {
        self.description
    }

    pub const fn r#type(&self) -> &MigrationType<'a, T> {
        &self.r#type
    }

    /// This function gets executed when the storage block height == the migration block height
    pub fn initialize(&self, storage: &mut T) {
        match &self.r#type {
            MigrationType::Regular(migration) => (migration.initialize_fn)(storage),
            _ => {
                tracing::trace!(
                    "Migration {} is not of type `Regular`, skipping",
                    self.name()
                )
            }
        }
    }

    /// This function gets executed when the storage block height >= the migration block height
    pub fn update(&self, storage: &mut T) {
        match &self.r#type {
            MigrationType::Regular(migration) => (migration.update_fn)(storage),
            _ => {
                tracing::trace!(
                    "Migration {} is not of type `Regular`, skipping",
                    self.name()
                )
            }
        }
    }

    /// This function gets executed when the storage block height == the migration block height
    pub fn hotfix<'b>(&'b self, b: &'b [u8]) -> Option<Vec<u8>> {
        match &self.r#type {
            MigrationType::Hotfix(migration) => (migration.hotfix_fn)(b),
            _ => {
                tracing::trace!(
                    "Migration {} is not of type `Hotfix`, skipping",
                    self.name()
                );
                None
            }
        }
    }
}

#[derive(Deserialize)]
struct IO<'a> {
    r#type: &'a str,

    #[serde(flatten)]
    metadata: Metadata,
}

pub fn load_migrations<'de: 'a, 'a, T: Clone>(
    registry: &[InnerMigration<'a, T>],
    data: &'a str,
) -> Result<BTreeMap<&'a str, Migration<'a, T>>, String> {
    // TODO: Do not hardcode the deserializer
    let config: Vec<IO> = serde_json::from_str(data).unwrap();

    // Build a BTreeMap from the linear registry
    let registry = registry
        .iter()
        .map(|m| (m.name, m))
        .collect::<BTreeMap<&'a str, &InnerMigration<'a, T>>>();

    Ok(config
        .into_iter()
        .map(|io| {
            let (&k, &v) = registry
                .get_key_value(io.r#type)
                .ok_or_else(|| format!("Unsupported migration type {}", io.r#type))?;
            Ok((k, Migration::new(v.clone(), io.metadata, Status::Enabled)))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?
        .into_iter()
        .collect())
}

/// Enable all migrations from the registry EXCEPT the hotfix
pub fn load_enable_all_regular_migrations<'a, T: Clone>(
    registry: &[InnerMigration<'a, T>],
) -> BTreeMap<&'a str, Migration<'a, T>> {
    registry
        .iter()
        .map(|m| {
            (
                m.name,
                Migration::new(
                    m.clone(),
                    Metadata::default(),
                    match m.r#type {
                        MigrationType::Regular(_) => Status::Enabled,
                        MigrationType::Hotfix(_) => Status::Disabled,
                    },
                ),
            )
        })
        .collect()
}
