use std::ops::Range;

use anyhow::anyhow;
use aoc_companion::prelude::*;
use itertools::Itertools;

pub(crate) struct Door(Vec<u32>);

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Door> {
        input
            .chars()
            .map(|c| {
                c.to_digit(10)
                    .ok_or_else(|| anyhow!("cannot parse {c:?} as digit"))
            })
            .try_collect()
            .map(Door)
    }

    fn part1(&self) -> usize {
        let mut disk = expand(&self.0);
        compact(&mut disk);
        checksum(&disk)
    }

    fn part2(&self) -> usize {
        let mut disk = expand(&self.0);
        defragment(&mut disk);
        checksum(&disk)
    }
}

fn expand(dense: &[u32]) -> Vec<Option<usize>> {
    dense
        .iter()
        .scan((true, 0), |(is_file, next_id), chunk| {
            let this_id = *next_id;
            let this_is_file = *is_file;
            *next_id += this_is_file as usize;
            *is_file = !*is_file;

            Some(std::iter::repeat_n(
                this_is_file.then_some(this_id),
                *chunk as usize,
            ))
        })
        .flatten()
        .collect()
}

fn compact(disk: &mut [Option<usize>]) {
    let mut front_idx = disk.iter().position(|d| d.is_none());
    let mut back_idx = disk.iter().rposition(|d| d.is_some());
    while let (Some(f), Some(b)) = (front_idx, back_idx) {
        if f >= b {
            break;
        }
        disk.swap(f, b);
        front_idx = disk[f..].iter().position(|d| d.is_none()).map(|x| f + x);
        back_idx = disk[..b].iter().rposition(|d| d.is_some());
    }
}

fn defragment(disk: &mut [Option<usize>]) {
    let Some(max_id) = disk.iter().rev().find_map(|x| *x) else {
        return;
    };

    for id in (0..=max_id).rev() {
        let file_span = find_span(Some(id), disk).unwrap();

        let (before, after) = disk.split_at_mut(file_span.start);

        let mut free_chunks = before
            .chunk_by_mut(|lhs, rhs| lhs.is_some() == rhs.is_some())
            .skip(1)
            .step_by(2);

        if let Some(free_chunk) = free_chunks.find(|chunk| chunk.len() >= file_span.len()) {
            let free_chunk = &mut free_chunk[..file_span.len()];
            let file_chunk = &mut after[..file_span.len()];
            free_chunk.swap_with_slice(file_chunk);
        }
    }
}

fn find_span(id: Option<usize>, disk: &[Option<usize>]) -> Option<Range<usize>> {
    let mut iter = disk.iter().enumerate().skip_while(|&(_, e)| *e != id);
    let (first, _) = iter.next()?;
    let len = iter.take_while(|(_, e)| **e == id).count() + 1;
    Some(first..(first + len))
}

fn checksum(disk: &[Option<usize>]) -> usize {
    disk.iter()
        .enumerate()
        .map(|(pos, id)| pos * id.unwrap_or(0))
        .sum()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use proptest::proptest;

    use super::*;

    const EXAMPLE_INPUT: &str = "2333133121414131402";

    #[test]
    fn compact_example_checksum() {
        let Door(disk) = Door::parse(EXAMPLE_INPUT).unwrap();
        let mut disk = expand(&disk);
        compact(&mut disk);
        assert_eq!(checksum(&disk), 1928);
    }

    #[test]
    fn defragmented_example_checksum() {
        let Door(disk) = Door::parse(EXAMPLE_INPUT).unwrap();
        let mut disk = expand(&disk);
        defragment(&mut disk);
        assert_eq!(checksum(&disk), 2858);
    }

    fn file_size_by_id(disk: &[Option<usize>]) -> HashMap<usize, usize> {
        let chunks = disk.iter().flatten().chunk_by(|x| **x);
        chunks
            .into_iter()
            .map(|(key, group)| (key, group.count()))
            .collect()
    }

    proptest! {

        #[test]
        fn defragmented_files_are_contiguous(input in "[1-9]([0-9][1-9])*") {
            let Door(disk) = Door::parse(&input).unwrap();
            let mut disk = expand(&disk);
            defragment(&mut disk);
            let chunks = disk.into_iter().flatten().chunk_by(|x| *x);
            assert!(chunks.into_iter().map(|(key, _)| key).duplicates().next().is_none());
        }

        #[test]
        fn defragment_keeps_files_at_same_size(input in "[1-9]([0-9][1-9])*") {
            let Door(disk) = Door::parse(&input).unwrap();
            let mut disk = expand(&disk);
            let file_sizes_before = file_size_by_id(&disk);
            defragment(&mut disk);
            let file_sizes_after = file_size_by_id(&disk);
            assert_eq!(file_sizes_before, file_sizes_after);
        }
    }
}
