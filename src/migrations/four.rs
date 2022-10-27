use crate::{Dummy, Storage, MIGRATION};
use linkme::distributed_slice;
use migrations_mre_linkme::InnerMigration;

fn hotfix(b: &[u8]) -> Option<Vec<u8>> {
    let mut d: Dummy = minicbor::decode(b).ok()?;
    d.0 = 10000000000;
    minicbor::to_vec(d).ok()
}

#[distributed_slice(MIGRATION)]
static FOUR: InnerMigration<Storage> = InnerMigration::new_hotfix(hotfix, "Four", "Wow!");
