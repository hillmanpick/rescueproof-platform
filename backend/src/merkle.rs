use sha3::{Digest, Keccak256};

// Double-hash leaves to separate leaf and internal node domains (OpenZeppelin compatible).
pub fn leaf(bytes: &[u8]) -> [u8; 32] {
    Keccak256::digest(Keccak256::digest(bytes)).into()
}
pub fn pair(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
    let (a, b) = if a <= b { (a, b) } else { (b, a) };
    let mut h = Keccak256::new();
    h.update(a);
    h.update(b);
    h.finalize().into()
}
pub type ProofTree = ([u8; 32], Vec<Vec<[u8; 32]>>);
pub fn build(leaves: &[[u8; 32]]) -> Option<ProofTree> {
    if leaves.is_empty() {
        return None;
    }
    let mut levels = vec![leaves.to_vec()];
    while levels.last()?.len() > 1 {
        let l = levels.last()?;
        let next = l
            .chunks(2)
            .map(|c| pair(c[0], *c.get(1).unwrap_or(&c[0])))
            .collect();
        levels.push(next);
    }
    let proofs = (0..leaves.len())
        .map(|mut i| {
            levels
                .iter()
                .take(levels.len() - 1)
                .map(|l| {
                    let sibling = *l.get(i ^ 1).unwrap_or(&l[i]);
                    i /= 2;
                    sibling
                })
                .collect()
        })
        .collect();
    Some((levels.last()?[0], proofs))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verifies_even_odd_and_single_trees() {
        for n in [1, 2, 3, 17] {
            let leaves: Vec<_> = (0..n).map(|i| leaf(i.to_string().as_bytes())).collect();
            let (root, proofs) = build(&leaves).unwrap();
            for (i, p) in proofs.iter().enumerate() {
                assert_eq!(p.iter().fold(leaves[i], |a, b| pair(a, *b)), root);
            }
            assert_ne!(leaf(b"tampered"), root);
        }
    }
    #[test]
    fn empty_has_no_root() {
        assert!(build(&[]).is_none());
    }
}
