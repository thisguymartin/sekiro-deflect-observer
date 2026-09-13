//! Read-only, bounded candidate-effect observations. See docs/reader-research.md.
use std::collections::BTreeSet;

pub const CANDIDATE_EFFECT: i32 = 105010;
pub const RESEARCH_HASH: &str = "637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856";
pub const MAX_NODES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadError {
    UnsupportedBuild,
    InvalidAddress,
    Unreadable,
    MissingPlayer,
    MissingEffects,
    Cycle,
    TooManyNodes,
    ChangedDuringRead,
    BudgetExceeded,
    InvalidAnimation,
    InvalidCamera,
}

impl ReadError {
    pub fn label(self) -> &'static str {
        match self {
            Self::UnsupportedBuild => "Unsupported executable",
            Self::InvalidAddress => "Invalid memory address",
            Self::Unreadable => "Memory read failed",
            Self::MissingPlayer => "Player unavailable (menu/loading)",
            Self::MissingEffects => "Player effects unavailable",
            Self::Cycle => "Memory list contains a cycle",
            Self::TooManyNodes => "Memory list exceeds limit",
            Self::ChangedDuringRead => "Ownership or animation changed during read",
            Self::BudgetExceeded => "Read exceeded time budget",
            Self::InvalidAnimation => "Enemy animation frame invalid",
            Self::InvalidCamera => "Camera pose or aspect invalid",
        }
    }
}

pub trait Memory {
    /// Must either fill the entire buffer or return an error, never dereference
    /// an untrusted game pointer directly. Native implementation also times out.
    fn read(&self, address: usize, bytes: &mut [u8]) -> Result<(), ReadError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub player: usize,
    pub effects: Vec<i32>,
}

impl Observation {
    pub fn candidate_present(&self) -> bool {
        self.effects.contains(&CANDIDATE_EFFECT)
    }
}

pub(crate) fn address(base: usize, offset: usize) -> Result<usize, ReadError> {
    let value = base.checked_add(offset).ok_or(ReadError::InvalidAddress)?;
    if base < 0x10000 || value > 0x0000_7fff_ffff_fff0 {
        return Err(ReadError::InvalidAddress);
    }
    Ok(value)
}

pub(crate) fn pointer(
    memory: &impl Memory,
    base: usize,
    offset: usize,
) -> Result<usize, ReadError> {
    let mut bytes = [0; 8];
    memory.read(address(base, offset)?, &mut bytes)?;
    let value = u64::from_le_bytes(bytes) as usize;
    if value != 0 && (value % 8 != 0 || address(value, 0).is_err()) {
        return Err(ReadError::InvalidAddress);
    }
    Ok(value)
}

fn chain(memory: &impl Memory, base: usize) -> Result<(usize, usize, usize), ReadError> {
    let world = pointer(memory, base, 0x3d7a1e0)?;
    if world == 0 {
        return Err(ReadError::MissingPlayer);
    }
    let player = pointer(memory, world, 0x88)?;
    if player == 0 {
        return Err(ReadError::MissingPlayer);
    }
    let effects = pointer(memory, player, 0x11d0)?;
    if effects == 0 {
        return Err(ReadError::MissingEffects);
    }
    Ok((world, player, effects))
}

fn list(memory: &impl Memory, manager: usize) -> Result<Vec<(usize, i32, usize)>, ReadError> {
    let mut node = pointer(memory, manager, 8)?;
    let mut visited = BTreeSet::new();
    let mut entries = Vec::new();
    while node != 0 {
        if !visited.insert(node) {
            return Err(ReadError::Cycle);
        }
        if entries.len() == MAX_NODES {
            return Err(ReadError::TooManyNodes);
        }
        let mut bytes = [0; 4];
        memory.read(address(node, 0x58)?, &mut bytes)?;
        let id = i32::from_le_bytes(bytes);
        let next = pointer(memory, node, 0x70)?;
        entries.push((node, id, next));
        node = next;
    }
    Ok(entries)
}

/// Hash gate runs before any game-state read. Two matching traversals plus
/// ownership rechecks reject observable mutation, but are not an atomic engine
/// snapshot and cannot eliminate pointer reuse between reads.
pub fn observe(memory: &impl Memory, base: usize, hash: &str) -> Result<Observation, ReadError> {
    if hash != RESEARCH_HASH {
        return Err(ReadError::UnsupportedBuild);
    }
    let initial = chain(memory, base)?;
    let first = list(memory, initial.2)?;
    if chain(memory, base)? != initial {
        return Err(ReadError::ChangedDuringRead);
    }
    let second = list(memory, initial.2)?;
    if first != second || chain(memory, base)? != initial {
        return Err(ReadError::ChangedDuringRead);
    }
    Ok(Observation {
        player: initial.1,
        effects: first.into_iter().map(|(_, id, _)| id).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::collections::BTreeMap;

    const BASE: usize = 0x140000000;
    const WORLD: usize = 0x20000;
    const PLAYER: usize = 0x30000;
    const MANAGER: usize = 0x40000;
    const NODE: usize = 0x50000;

    #[derive(Default)]
    struct Fixture {
        bytes: BTreeMap<usize, u8>,
        calls: Cell<usize>,
        change_player: bool,
        change_effect: bool,
    }
    impl Memory for Fixture {
        fn read(&self, address: usize, bytes: &mut [u8]) -> Result<(), ReadError> {
            self.calls.set(self.calls.get() + 1);
            if self.change_effect && address == NODE + 0x58 && self.calls.get() > 6 {
                bytes.copy_from_slice(&7i32.to_le_bytes());
                return Ok(());
            }
            if self.change_player && address == WORLD + 0x88 && self.calls.get() > 6 {
                bytes.copy_from_slice(&0x90000u64.to_le_bytes());
                return Ok(());
            }
            for (i, byte) in bytes.iter_mut().enumerate() {
                *byte = *self
                    .bytes
                    .get(&(address + i))
                    .ok_or(ReadError::Unreadable)?;
            }
            Ok(())
        }
    }
    impl Fixture {
        fn put(&mut self, address: usize, bytes: &[u8]) {
            for (i, byte) in bytes.iter().enumerate() {
                self.bytes.insert(address + i, *byte);
            }
        }
        fn ptr(&mut self, address: usize, value: usize) {
            self.put(address, &(value as u64).to_le_bytes());
        }
        fn new(ids: &[i32]) -> Self {
            let mut f = Self::default();
            f.ptr(BASE + 0x3d7a1e0, WORLD);
            f.ptr(WORLD + 0x88, PLAYER);
            f.ptr(PLAYER + 0x11d0, MANAGER);
            f.ptr(MANAGER + 8, if ids.is_empty() { 0 } else { NODE });
            for (i, id) in ids.iter().enumerate() {
                let node = NODE + i * 0x100;
                f.put(node + 0x58, &id.to_le_bytes());
                f.ptr(
                    node + 0x70,
                    if i + 1 == ids.len() { 0 } else { node + 0x100 },
                );
            }
            f
        }
        fn observe(&self) -> Result<Observation, ReadError> {
            observe(self, BASE, RESEARCH_HASH)
        }
    }

    #[test]
    fn unsupported_build_performs_zero_reads() {
        let f = Fixture::default();
        assert_eq!(
            observe(&f, BASE, "unknown"),
            Err(ReadError::UnsupportedBuild)
        );
        assert_eq!(f.calls.get(), 0);
    }
    #[test]
    fn complete_lists_distinguish_presence_and_valid_empty() {
        for ids in [vec![], vec![5], vec![5, CANDIDATE_EFFECT, 7]] {
            let result = Fixture::new(&ids).observe().unwrap();
            assert_eq!(result.candidate_present(), ids.contains(&CANDIDATE_EFFECT));
            assert_eq!(result.effects, ids);
            assert_eq!(result.player, PLAYER);
        }
    }
    #[test]
    fn failed_read_at_every_required_byte_is_unknown() {
        let original = Fixture::new(&[CANDIDATE_EFFECT]);
        for address in original.bytes.keys() {
            let mut f = Fixture::new(&[CANDIDATE_EFFECT]);
            f.bytes.remove(address);
            assert_eq!(f.observe(), Err(ReadError::Unreadable), "{address:x}");
        }
    }
    #[test]
    fn cycles_and_over_limit_lists_never_report_present() {
        let mut f = Fixture::new(&[CANDIDATE_EFFECT]);
        f.ptr(NODE + 0x70, NODE);
        assert_eq!(f.observe(), Err(ReadError::Cycle));
        assert_eq!(
            Fixture::new(&vec![CANDIDATE_EFFECT; MAX_NODES + 1]).observe(),
            Err(ReadError::TooManyNodes)
        );
        assert!(Fixture::new(&vec![1; MAX_NODES]).observe().is_ok());
    }
    #[test]
    fn missing_player_invalid_pointer_and_player_change_are_unknown() {
        let mut f = Fixture::new(&[CANDIDATE_EFFECT]);
        f.ptr(WORLD + 0x88, 0);
        assert_eq!(f.observe(), Err(ReadError::MissingPlayer));
        f.ptr(WORLD + 0x88, usize::MAX);
        assert_eq!(f.observe(), Err(ReadError::InvalidAddress));
        let mut f = Fixture::new(&[CANDIDATE_EFFECT]);
        f.ptr(0x90000 + 0x11d0, MANAGER);
        f.change_player = true;
        assert_eq!(f.observe(), Err(ReadError::ChangedDuringRead));
    }

    #[test]
    fn effect_mutation_between_traversals_is_unknown() {
        let mut f = Fixture::new(&[CANDIDATE_EFFECT]);
        f.change_effect = true;
        assert_eq!(f.observe(), Err(ReadError::ChangedDuringRead));
    }
}
