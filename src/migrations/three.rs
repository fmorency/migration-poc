use crate::{Dummy, Storage, MIG};
use linkme::distributed_slice;
use migrations_mre_linkme::InnerMig;

fn hotfix(b: &[u8]) -> Option<Vec<u8>> {
    let mut d: Dummy = minicbor::decode(b).ok()?;
    d.0 = 12345;
    minicbor::to_vec(d).ok()
}

fn desc() -> (&'static str, &'static str) {
    ("Three", "Some cool hotfix")
}

#[distributed_slice(MIG)]
static THREE: InnerMig<Storage> = InnerMig::new_run(hotfix, desc);
