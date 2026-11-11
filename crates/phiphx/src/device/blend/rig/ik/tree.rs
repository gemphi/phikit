//! Gathering IK constraints into trees at their chain roots (`initialize_posetree`).
use crate::device::blend::rig::ik::*;

impl BRig {
    pub(super) fn ik_data(&self, bone: usize, con: usize) -> Option<&BIkData> {
        match &self.bones[bone].constraints[con].kind {
            BConstraintKind::Ik(d) => Some(d),
            _ => None,
        }
    }

    /// `iksolver_initialize_tree`: every IK constraint's chain, gathered
    /// into trees at their roots, in evaluation order; and each chain
    /// bone's tree.
    pub fn ik_trees(&self, state: &BRigState) -> (Vec<BIkTree>, Vec<Option<usize>>) {
        let mut by_root: Vec<(usize, Vec<BIkTree>)> = Vec::new();
        for owner in 0..self.bones.len() {
            let cons: Vec<usize> = (0..self.bones[owner].constraints.len())
                .filter(|&c| {
                    let con = &self.bones[owner].constraints[c];
                    let Some(d) = self.ik_data(owner, c) else { return false };
                    // `find_ik_constraints`.
                    d.flag & IK_AUTO != 0 || (!matches!(d.target, super::super::constraint::BTarget::None | super::super::constraint::BTarget::Rig(None)) && con.flag & CONSTRAINT_DISABLE == 0)
                })
                .collect();
            let mut tip = owner;
            for c in cons {
                let d = self.ik_data(owner, c).expect("an IK constraint");
                if d.flag & IK_TIP == 0 {
                    if let Some(p) = self.bones[tip].parent {
                        tip = p;
                    }
                }
                let mut chain = Vec::new();
                let mut cur = Some(tip);
                while let Some(b) = cur {
                    chain.push(b);
                    if chain.len() as i32 == d.rootbone || chain.len() > 255 {
                        break;
                    }
                    cur = self.bones[b].parent;
                }
                let root = *chain.last().expect("a chain");
                let trees = match by_root.iter().position(|(r, _)| *r == root) {
                    Some(i) => &mut by_root[i].1,
                    None => {
                        by_root.push((root, Vec::new()));
                        &mut by_root.last_mut().expect("just pushed").1
                    }
                };
                // A tree some of whose targets lie off this chain takes it.
                let mut in_chain: Vec<usize> = chain.clone();
                let found = trees.iter().position(|tree| {
                    tree.targets.iter().any(|t| {
                        let b = tree.bones[t.tip];
                        match in_chain.iter().position(|&x| x == b) {
                            Some(i) => {
                                in_chain.remove(i);
                                false
                            }
                            None => true,
                        }
                    })
                });
                let seg = chain.len();
                let target_tip;
                match found {
                    None => {
                        let bones: Vec<usize> = chain.iter().rev().copied().collect();
                        let parent = (0..seg as i32).map(|a| a - 1).collect();
                        target_tip = seg - 1;
                        trees.push(BIkTree { bones, parent, targets: Vec::new(), iterations: d.iterations, stretch: d.flag & IK_STRETCH != 0 });
                    }
                    Some(i) => {
                        let mut tree = trees.remove(i);
                        tree.iterations = tree.iterations.max(d.iterations);
                        tree.stretch = tree.stretch && d.flag & IK_STRETCH == 0;
                        let size = seg.min(tree.bones.len());
                        let (mut a, mut t) = (0, 0);
                        while a < size && t < tree.bones.len() {
                            while t < tree.bones.len() && tree.bones[t] != chain[seg - a - 1] {
                                t += 1;
                            }
                            if t >= tree.bones.len() {
                                break;
                            }
                            while a < size && t < tree.bones.len() && tree.bones[t] == chain[seg - a - 1] {
                                a += 1;
                                t += 1;
                            }
                        }
                        let rest = seg - a;
                        target_tip = tree.bones.len() + rest - 1;
                        if rest > 0 {
                            let total = tree.bones.len();
                            let first_parent = self.bones[chain[rest - 1]].parent;
                            let parent = (a as i32 - 1).max(0) as usize;
                            let parent = (parent..total).find(|&p| Some(tree.bones[p]) == first_parent).unwrap_or(a.saturating_sub(1));
                            for k in 0..rest {
                                tree.bones.push(chain[rest - k - 1]);
                                tree.parent.push((total + k) as i32 - 1);
                            }
                            tree.parent[total] = parent as i32;
                        }
                        trees.push(tree);
                    }
                }
                let tree = trees.last_mut().expect("a tree");
                tree.targets.push(BIkTarget { bone: owner, con: c, tip: target_tip });
                // One active IK constraint per bone.
                if state.influence[owner][c] != 0.0 && !state.muted[owner][c] {
                    break;
                }
            }
        }
        let trees: Vec<BIkTree> = by_root.into_iter().flat_map(|(_, t)| t).collect();
        let mut tree_of = vec![None; self.bones.len()];
        for (i, t) in trees.iter().enumerate() {
            for &b in &t.bones {
                tree_of[b].get_or_insert(i);
            }
        }
        (trees, tree_of)
    }
}
