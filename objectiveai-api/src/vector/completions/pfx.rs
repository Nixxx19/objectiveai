use indexmap::IndexMap;
use rand::{Rng, seq::SliceRandom};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pfx {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
}

impl Pfx {
    pub fn to_char(&self) -> char {
        match self {
            Pfx::A => 'A',
            Pfx::B => 'B',
            Pfx::C => 'C',
            Pfx::D => 'D',
            Pfx::E => 'E',
            Pfx::F => 'F',
            Pfx::G => 'G',
            Pfx::H => 'H',
            Pfx::I => 'I',
            Pfx::J => 'J',
            Pfx::K => 'K',
            Pfx::L => 'L',
            Pfx::M => 'M',
            Pfx::N => 'N',
            Pfx::O => 'O',
            Pfx::P => 'P',
            Pfx::Q => 'Q',
            Pfx::R => 'R',
            Pfx::S => 'S',
            Pfx::T => 'T',
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'A' => Some(Pfx::A),
            'B' => Some(Pfx::B),
            'C' => Some(Pfx::C),
            'D' => Some(Pfx::D),
            'E' => Some(Pfx::E),
            'F' => Some(Pfx::F),
            'G' => Some(Pfx::G),
            'H' => Some(Pfx::H),
            'I' => Some(Pfx::I),
            'J' => Some(Pfx::J),
            'K' => Some(Pfx::K),
            'L' => Some(Pfx::L),
            'M' => Some(Pfx::M),
            'N' => Some(Pfx::N),
            'O' => Some(Pfx::O),
            'P' => Some(Pfx::P),
            'Q' => Some(Pfx::Q),
            'R' => Some(Pfx::R),
            'S' => Some(Pfx::S),
            'T' => Some(Pfx::T),
            _ => None,
        }
    }

    pub fn rng_vec(rng: &mut impl Rng) -> Vec<Self> {
        let mut vec = vec![
            Pfx::A,
            Pfx::B,
            Pfx::C,
            Pfx::D,
            Pfx::E,
            Pfx::F,
            Pfx::G,
            Pfx::H,
            Pfx::I,
            Pfx::J,
            Pfx::K,
            Pfx::L,
            Pfx::M,
            Pfx::N,
            Pfx::O,
            Pfx::P,
            Pfx::Q,
            Pfx::R,
            Pfx::S,
            Pfx::T,
        ];
        vec.shuffle(rng);
        vec
    }
}

impl std::fmt::Display for Pfx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Debug, Clone)]
pub enum PfxTree {
    Branch(Arc<IndexMap<Pfx, PfxTree>>),
    Leaf(usize),
}

impl PfxTree {
    pub fn new(
        rng: &mut impl Rng,
        source_len: usize,
        max_branch_len: usize,
    ) -> Self {
        let mut source: Vec<usize> = (0..source_len).collect();
        source.shuffle(rng);
        Self::new_inner(rng, &source, max_branch_len, false)
    }

    pub fn new_inner(
        rng: &mut impl Rng,
        source: &[usize],
        max_branch_len: usize,
        force_sub_branch: bool,
    ) -> Self {
        let pfxs = Pfx::rng_vec(rng);
        if !force_sub_branch && source.len() <= max_branch_len {
            // return a single branch containing all leaves
            let mut branch = IndexMap::with_capacity(source.len());
            for (i, source_index) in source.iter().enumerate() {
                branch.insert(pfxs[i], PfxTree::Leaf(*source_index));
            }
            Self::Branch(Arc::new(branch))
        } else {
            // split into sub-branches
            let n = {
                let candidate =
                    (source.len() + max_branch_len - 1) / max_branch_len;
                if candidate <= max_branch_len {
                    candidate
                } else {
                    max_branch_len
                }
            };
            let base_per = source.len() / n;
            let extra = source.len() % n;
            let force_sub_branch =
                base_per + { if extra > 0 { 1 } else { 0 } } > max_branch_len;
            let mut branch = IndexMap::with_capacity(n);
            let mut i = 0;
            let mut count = 0;
            while i < n {
                let branch_len = base_per + if i < extra { 1 } else { 0 };
                branch.insert(
                    pfxs[i],
                    PfxTree::new_inner(
                        rng,
                        &source[count..count + branch_len],
                        max_branch_len,
                        force_sub_branch,
                    ),
                );
                count += branch_len;
                i += 1;
            }
            Self::Branch(Arc::new(branch))
        }
    }

    pub fn pfx_indices(
        &self,
        rng: &mut impl Rng,
        source_len: usize,
    ) -> Vec<(String, usize)> {
        let mut indices = Vec::with_capacity(source_len);
        self.pfx_indices_inner(None, &mut indices);
        indices.shuffle(rng);
        indices
    }

    pub fn pfx_indices_inner(
        &self,
        parent_pfx: Option<String>,
        indices: &mut Vec<(String, usize)>,
    ) {
        match self {
            PfxTree::Branch(branch) => {
                for (pfx, child) in branch.as_ref() {
                    let parent_pfx = Some(match &parent_pfx {
                        Some(parent_pfx) => format!("{}`{}`", parent_pfx, pfx),
                        None => format!("`{}`", pfx),
                    });
                    child.pfx_indices_inner(parent_pfx, indices);
                }
            }
            PfxTree::Leaf(index) => {
                indices.push((parent_pfx.unwrap(), *index));
            }
        }
    }

    pub fn get(&self, pfx: Pfx) -> Option<PfxTree> {
        match self {
            PfxTree::Branch(branch) => branch.get(&pfx).cloned(),
            PfxTree::Leaf(_) => None,
        }
    }

    pub fn depth(&self) -> usize {
        match self {
            PfxTree::Branch(branch) => {
                1 + branch
                    .values()
                    .next() // all sub-branches have the same depth
                    .map(|v| v.depth())
                    .unwrap_or(0)
            }
            PfxTree::Leaf(_) => 0,
        }
    }

    pub fn unwrap_leaf(&self) -> usize {
        match self {
            PfxTree::Leaf(index) => *index,
            PfxTree::Branch(_) => {
                panic!("Called unwrap_leaf on a Branch")
            }
        }
    }

    pub fn regex_patterns(&self, keys: &[(String, usize)]) -> (String, String) {
        let depth = self.depth();
        let mut with_ticks = String::with_capacity(
            (keys.len() - 1) // '|' characters
                + (keys.len() * depth * 3) // each key
                + keys.len() * 2, // parentheses
        );
        let mut without_ticks = String::with_capacity(
            (keys.len() - 1) // for '|' characters
                + keys.len() * (depth * 3 - 2) // each key stripped of ticks
                + keys.len() * 2, // parentheses
        );
        for (key, _) in keys {
            if with_ticks.len() > 0 {
                with_ticks.push('|');
                without_ticks.push('|');
            }
            with_ticks.push('(');
            without_ticks.push('(');
            with_ticks.push_str(key);
            without_ticks.push_str(&key[1..key.len() - 1]); // strip ticks
            with_ticks.push(')');
            without_ticks.push(')');
        }
        (with_ticks, without_ticks)
    }
}

#[derive(Debug, Clone)]
pub struct PfxData {
    pub pfx_tree: PfxTree,
    pub responses_key_pattern: String,
    pub responses_key_pattern_stripped: String,
}
