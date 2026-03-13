use crate::canonical::digest::sha256_hex;

pub fn canonical_tree_hash<'a, I>(entries: I) -> String
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let mut ordered: Vec<(&str, &str)> = entries.into_iter().collect();
    ordered.sort_unstable_by(|left, right| left.0.cmp(right.0));

    let mut material = Vec::new();
    for (path, digest) in ordered {
        material.extend_from_slice(path.as_bytes());
        material.push(0);
        material.extend_from_slice(digest.as_bytes());
        material.push(0);
    }

    sha256_hex(&material)
}
