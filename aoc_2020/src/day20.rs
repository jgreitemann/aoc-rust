use std::collections::{HashMap, HashSet};

use anyhow::{Context, anyhow, bail};
use aoc_companion::prelude::*;
use itertools::Itertools;
use ndarray::s;

pub(crate) struct Door {
    tiles: HashMap<TileId, Tile>,
}

impl<'input> Solution<'input> for Door {
    fn parse(input: &'input str) -> Result<Self> {
        input
            .split("\n\n")
            .map(|block| {
                let Some((preamble, tile)) = block.split_once(':') else {
                    bail!("missing colon after tile preample");
                };
                let tile_id = preamble
                    .strip_prefix("Tile ")
                    .ok_or_else(|| anyhow!("missing tile introducer"))?
                    .parse()
                    .context("failed to parse tile ID")
                    .map(TileId)?;
                let tile = aoc_utils::geometry::try_parse_map(tile.trim(), |b| match b {
                    b'.' => Ok(0),
                    b'#' => Ok(1),
                    _ => Err(InvalidTileChar { byte: b }),
                })
                .context("")?;
                Ok((tile_id, tile))
            })
            .try_collect()
            .map(|tiles| Self { tiles })
    }

    fn part1(&self) -> u64 {
        corner_tiles(
            tiles_by_edge_signature(self.tiles.iter().map(|(&id, tile)| (id, tile.view())))
                .values(),
        )
        .map(|TileId(id)| id)
        .product()
    }

    fn part2(&self) -> Result<usize> {
        Ok(
            purge_monsters(puzzle(self.tiles.iter().map(|(&id, tile)| (id, tile.view()))).view())?
                .into_iter()
                .filter(|&x| x == 1)
                .count(),
        )
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid tile character {:?} ({byte:x})", String::from_utf8_lossy(&[self.byte]))]
struct InvalidTileChar {
    byte: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TileId(u64);

type Tile = ndarray::Array2<u8>;
type TileView<'a> = ndarray::ArrayView2<'a, u8>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct EdgeSignature(u16);

impl EdgeSignature {
    fn new<'a>(edge: impl IntoIterator<Item = &'a u8>) -> Self {
        let sig = edge.into_iter().fold(0, |acc, e| (acc << 1) | *e as u16);
        Self(sig.min(sig.reverse_bits() >> 6))
    }
}

fn edge_signatures(tile: TileView) -> [EdgeSignature; 4] {
    [
        EdgeSignature::new(tile.slice(s!(.., 0))),
        EdgeSignature::new(tile.slice(s!(0, ..))),
        EdgeSignature::new(tile.slice(s!(.., -1))),
        EdgeSignature::new(tile.slice(s!(-1, ..))),
    ]
}

fn tiles_by_edge_signature<'a>(
    tiles: impl IntoIterator<Item = (TileId, TileView<'a>)>,
) -> HashMap<EdgeSignature, Vec<TileId>> {
    tiles
        .into_iter()
        .flat_map(|(tile_id, tile)| edge_signatures(tile).map(|e| (e, tile_id)))
        .into_group_map()
}

fn corner_tiles<'a>(
    tiles_with_same_edges: impl IntoIterator<Item = &'a Vec<TileId>>,
) -> impl Iterator<Item = TileId> {
    tiles_with_same_edges
        .into_iter()
        .filter_map(|v| match v[..] {
            [tile_id] => Some(tile_id),
            _ => None,
        })
        .duplicates()
}

fn border_tiles<'a>(
    tiles_with_same_edges: impl IntoIterator<Item = &'a Vec<TileId>>,
) -> HashSet<TileId> {
    tiles_with_same_edges
        .into_iter()
        .filter_map(|v| match v[..] {
            [tile_id] => Some(tile_id),
            _ => None,
        })
        .collect()
}

fn shares_edge_with(edge: EdgeSignature, tile: TileView) -> bool {
    edge_signatures(tile).into_iter().any(|e| e == edge)
}

fn puzzle<'a>(tiles: impl IntoIterator<Item = (TileId, TileView<'a>)>) -> ndarray::Array2<u8> {
    let tiles = HashMap::<_, _>::from_iter(tiles);
    let tiles_by_edges = tiles_by_edge_signature(tiles.iter().map(|(&k, &v)| (k, v)));

    let dim = tiles.len().isqrt();
    let mut puzzled_ids = ndarray::Array2::from_elem((dim, dim), None);

    // Identify corner and border tiles based on edge-sharing statistics
    let top_left_corner = corner_tiles(tiles_by_edges.values()).next().unwrap();
    let mut border_tiles = border_tiles(tiles_by_edges.values());

    // Trace out the border tiles in order
    border_tiles.remove(&top_left_corner);
    let mut border_ids = std::iter::successors(Some(top_left_corner), |prev| {
        let prev_edges = edge_signatures(tiles[prev]);
        border_tiles
            .extract_if(|b| {
                prev_edges
                    .iter()
                    .any(|&edge| shares_edge_with(edge, tiles[b]))
            })
            .next()
    });

    // Spread the border tile IDs on the square border
    for slice in [s![0, ..-1], s![..-1, -1], s![-1,1..;-1], s![1..;-1,0]] {
        for (dest, source) in puzzled_ids.slice_mut(slice).iter_mut().zip(&mut border_ids) {
            *dest = Some(source);
        }
    }

    // Fill in the inner tiles based two neighboring known tiles
    for (i, j) in (0..dim - 2).cartesian_product(0..dim - 2) {
        let upper_edges = edge_signatures(tiles[&puzzled_ids[(i, j + 1)].unwrap()]);
        let left_edges = edge_signatures(tiles[&puzzled_ids[(i + 1, j)].unwrap()]);
        puzzled_ids[(i + 1, j + 1)] = Some(
            *tiles
                .iter()
                .filter(|(id, _)| **id != puzzled_ids[(i, j)].unwrap())
                .find(|(_, tile)| {
                    upper_edges
                        .iter()
                        .any(|&edge| shares_edge_with(edge, **tile))
                        && left_edges
                            .iter()
                            .any(|&edge| shares_edge_with(edge, **tile))
                })
                .unwrap()
                .0,
        );
    }

    // Place matching rotoreflection of tile in the final grid
    let cell_dim = tiles.values().next().unwrap().dim().0 - 2;
    let grid_dim = cell_dim * dim;
    let mut grid = ndarray::Array2::from_elem((grid_dim, grid_dim), 0);

    dbg!(puzzled_ids.clone().map(|t| t.unwrap().0));

    for (((i, j), id), mut dest) in puzzled_ids
        .indexed_iter()
        .zip_eq(grid.exact_chunks_mut((cell_dim, cell_dim)))
    {
        let id = id.unwrap();
        let fitting_rotoreflection = rotoreflections(tiles[&id])
            .iter()
            .copied()
            .filter(|&rr| {
                if i > 0 {
                    shares_edge_with(
                        edge_signatures(rr)[1],
                        tiles[&puzzled_ids[(i - 1, j)].unwrap()],
                    )
                } else {
                    shares_edge_with(
                        edge_signatures(rr)[3],
                        tiles[&puzzled_ids[(i + 1, j)].unwrap()],
                    )
                }
            })
            .find(|&rr| {
                if j > 0 {
                    shares_edge_with(
                        edge_signatures(rr)[0],
                        tiles[&puzzled_ids[(i, j - 1)].unwrap()],
                    )
                } else {
                    shares_edge_with(
                        edge_signatures(rr)[2],
                        tiles[&puzzled_ids[(i, j + 1)].unwrap()],
                    )
                }
            })
            .with_context(|| format!("failed to find fitting rotoreflection for {:?}", (i, j)))
            .unwrap();

        dest.assign(
            &fitting_rotoreflection
                .slice(s![1.., 1..])
                .slice(s![..-1, ..-1]),
        );
    }

    grid
}

fn rotoreflections(tile: TileView) -> [TileView; 8] {
    let mut swapped = tile;
    swapped.swap_axes(0, 1);
    [
        tile.slice_move(s![.., ..]),
        tile.slice_move(s![..;-1,..]),
        tile.slice_move(s![..,..;-1]),
        tile.slice_move(s![..;-1,..;-1]),
        swapped.slice_move(s![.., ..]),
        swapped.slice_move(s![..;-1,..]),
        swapped.slice_move(s![..,..;-1]),
        swapped.slice_move(s![..;-1,..;-1]),
    ]
}

fn purge_monsters(puzzle: TileView) -> Result<Tile> {
    const MONSTER_MASK: TileView = ndarray::aview2(&[
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0],
        [1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1],
        [0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0],
    ]);

    let Some(puzzle) = rotoreflections(puzzle).iter().copied().find(|puzzle| {
        puzzle
            .windows(MONSTER_MASK.dim())
            .into_iter()
            .any(|window| &window * &MONSTER_MASK == MONSTER_MASK)
    }) else {
        bail!("did not find any sea monsters");
    };

    let mut monsters = Tile::from_elem(puzzle.dim(), 0);
    for (puzzle_win, monsters_win) in puzzle
        .windows(MONSTER_MASK.dim())
        .into_iter()
        .zip_eq(monsters.cell_view().windows(MONSTER_MASK.dim()))
    {
        if &puzzle_win * &MONSTER_MASK == MONSTER_MASK {
            ndarray::Zip::from(monsters_win)
                .and(MONSTER_MASK)
                .for_each(|dest, mask| dest.update(|d| d.max(*mask)));
        }
    }

    Ok(monsters ^ puzzle)
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use itertools::Itertools;

    use super::*;

    #[test]
    fn parse_example_input() {
        let Door { tiles } = Door::parse(EXAMPLE_INPUT).unwrap();
        let expected_tiles = HashMap::from(EXAMPLE_TILES);
        for tile_id in tiles.keys().chain(expected_tiles.keys()).unique() {
            assert_eq!(
                tiles.get(tile_id).map(|a| a.view()).as_ref(),
                expected_tiles.get(tile_id),
                "mismatch for {tile_id:?}"
            )
        }
    }

    #[test]
    fn edge_signatures_of_example_tile_2311() {
        assert_eq!(
            HashSet::from(edge_signatures(EXAMPLE_TILES[0].1)),
            HashSet::from(TILE_2311_EDGE_SIGNATURES)
        );
    }

    #[test]
    fn rotoreflections_have_the_same_edge_signatures() {
        for rotoreflection in rotoreflections(EXAMPLE_TILES[0].1) {
            assert_eq!(
                HashSet::from(edge_signatures(rotoreflection)),
                HashSet::from(TILE_2311_EDGE_SIGNATURES)
            );
        }
    }

    #[test]
    fn example_tiles_by_edge_signature() {
        assert_eq!(
            tiles_by_edge_signature(EXAMPLE_TILES),
            HashMap::from([
                // corner tiles
                (EdgeSignature(391), vec![TileId(1171)]),
                (EdgeSignature(24), vec![TileId(1171)]),
                (EdgeSignature(587), vec![TileId(1951)]),
                (EdgeSignature(177), vec![TileId(1951)]),
                (EdgeSignature(78), vec![TileId(2971)]),
                (EdgeSignature(161), vec![TileId(2971)]),
                (EdgeSignature(501), vec![TileId(3079)]),
                (EdgeSignature(66), vec![TileId(3079)]),
                // border tiles
                (EdgeSignature(43), vec![TileId(1489)]),
                (EdgeSignature(481), vec![TileId(2473)]),
                (EdgeSignature(231), vec![TileId(2311)]),
                (EdgeSignature(271), vec![TileId(2729)]),
                // inner tiles
                (EdgeSignature(234), vec![TileId(1427), TileId(2473)]),
                (EdgeSignature(85), vec![TileId(2971), TileId(2729)]),
                (EdgeSignature(399), vec![TileId(1171), TileId(2473)]),
                (EdgeSignature(9), vec![TileId(1427), TileId(2729)]),
                (EdgeSignature(318), vec![TileId(2311), TileId(1951)]),
                (EdgeSignature(89), vec![TileId(2311), TileId(3079)]),
                (EdgeSignature(210), vec![TileId(2311), TileId(1427)]),
                (EdgeSignature(18), vec![TileId(1171), TileId(1489)]),
                (EdgeSignature(565), vec![TileId(1489), TileId(2971)]),
                (EdgeSignature(116), vec![TileId(2473), TileId(3079)]),
                (EdgeSignature(183), vec![TileId(1427), TileId(1489)]),
                (EdgeSignature(397), vec![TileId(1951), TileId(2729)]),
            ])
        );
    }

    #[test]
    fn identify_example_corner_tiles() {
        assert_eq!(
            HashSet::from_iter(corner_tiles(
                tiles_by_edge_signature(EXAMPLE_TILES).values()
            )),
            HashSet::from([TileId(1171), TileId(1951), TileId(2971), TileId(3079)])
        );
    }

    #[test]
    fn identify_example_border_tiles() {
        assert_eq!(
            border_tiles(tiles_by_edge_signature(EXAMPLE_TILES).values()),
            HashSet::from([
                TileId(1951),
                TileId(2311),
                TileId(3079),
                TileId(2473),
                TileId(1171),
                TileId(1489),
                TileId(2971),
                TileId(2729),
            ])
        );
    }

    #[test]
    fn solve_example_puzzle() {
        assert!(
            rotoreflections(puzzle(EXAMPLE_TILES).view())
                .iter()
                .contains(&EXAMPLE_PUZZLE)
        );
    }

    #[test]
    fn purge_monsters_from_example() {
        let expected: Tile =
            aoc_utils::geometry::try_parse_map::<_, Infallible>(PURGED_EXAMPLE, |b| {
                Ok((b == b'#') as u8)
            })
            .unwrap();
        for puzzle_roto in rotoreflections(EXAMPLE_PUZZLE) {
            assert_eq!(purge_monsters(puzzle_roto).unwrap(), expected);
        }
    }

    const TILE_2311_EDGE_SIGNATURES: [EdgeSignature; 4] = [
        EdgeSignature(0b0100111110),
        EdgeSignature(0b0001011001),
        EdgeSignature(0b0011010010),
        EdgeSignature(0b0011100111),
    ];

    const EXAMPLE_INPUT: &str = "\
Tile 2311:
..##.#..#.
##..#.....
#...##..#.
####.#...#
##.##.###.
##...#.###
.#.#.#..##
..#....#..
###...#.#.
..###..###

Tile 1951:
#.##...##.
#.####...#
.....#..##
#...######
.##.#....#
.###.#####
###.##.##.
.###....#.
..#.#..#.#
#...##.#..

Tile 1171:
####...##.
#..##.#..#
##.#..#.#.
.###.####.
..###.####
.##....##.
.#...####.
#.##.####.
####..#...
.....##...

Tile 1427:
###.##.#..
.#..#.##..
.#.##.#..#
#.#.#.##.#
....#...##
...##..##.
...#.#####
.#.####.#.
..#..###.#
..##.#..#.

Tile 1489:
##.#.#....
..##...#..
.##..##...
..#...#...
#####...#.
#..#.#.#.#
...#.#.#..
##.#...##.
..##.##.##
###.##.#..

Tile 2473:
#....####.
#..#.##...
#.##..#...
######.#.#
.#...#.#.#
.#########
.###.#..#.
########.#
##...##.#.
..###.#.#.

Tile 2971:
..#.#....#
#...###...
#.#.###...
##.##..#..
.#####..##
.#..####.#
#..#.#..#.
..####.###
..#.#.###.
...#.#.#.#

Tile 2729:
...#.#.#.#
####.#....
..#.#.....
....#..#.#
.##..##.#.
.#.####...
####.#.#..
##.####...
##..#.##..
#.##...##.

Tile 3079:
#.#.#####.
.#..######
..#.......
######....
####.#..#.
.#...#.##.
#.#####.##
..#.###...
..#.......
..#.###...";

    const EXAMPLE_TILES: [(TileId, TileView); 9] = [
        (
            TileId(2311),
            ndarray::aview2(&[
                [0, 0, 1, 1, 0, 1, 0, 0, 1, 0],
                [1, 1, 0, 0, 1, 0, 0, 0, 0, 0],
                [1, 0, 0, 0, 1, 1, 0, 0, 1, 0],
                [1, 1, 1, 1, 0, 1, 0, 0, 0, 1],
                [1, 1, 0, 1, 1, 0, 1, 1, 1, 0],
                [1, 1, 0, 0, 0, 1, 0, 1, 1, 1],
                [0, 1, 0, 1, 0, 1, 0, 0, 1, 1],
                [0, 0, 1, 0, 0, 0, 0, 1, 0, 0],
                [1, 1, 1, 0, 0, 0, 1, 0, 1, 0],
                [0, 0, 1, 1, 1, 0, 0, 1, 1, 1],
            ]),
        ),
        (
            TileId(1951),
            ndarray::aview2(&[
                [1, 0, 1, 1, 0, 0, 0, 1, 1, 0],
                [1, 0, 1, 1, 1, 1, 0, 0, 0, 1],
                [0, 0, 0, 0, 0, 1, 0, 0, 1, 1],
                [1, 0, 0, 0, 1, 1, 1, 1, 1, 1],
                [0, 1, 1, 0, 1, 0, 0, 0, 0, 1],
                [0, 1, 1, 1, 0, 1, 1, 1, 1, 1],
                [1, 1, 1, 0, 1, 1, 0, 1, 1, 0],
                [0, 1, 1, 1, 0, 0, 0, 0, 1, 0],
                [0, 0, 1, 0, 1, 0, 0, 1, 0, 1],
                [1, 0, 0, 0, 1, 1, 0, 1, 0, 0],
            ]),
        ),
        (
            TileId(1171),
            ndarray::aview2(&[
                [1, 1, 1, 1, 0, 0, 0, 1, 1, 0],
                [1, 0, 0, 1, 1, 0, 1, 0, 0, 1],
                [1, 1, 0, 1, 0, 0, 1, 0, 1, 0],
                [0, 1, 1, 1, 0, 1, 1, 1, 1, 0],
                [0, 0, 1, 1, 1, 0, 1, 1, 1, 1],
                [0, 1, 1, 0, 0, 0, 0, 1, 1, 0],
                [0, 1, 0, 0, 0, 1, 1, 1, 1, 0],
                [1, 0, 1, 1, 0, 1, 1, 1, 1, 0],
                [1, 1, 1, 1, 0, 0, 1, 0, 0, 0],
                [0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
            ]),
        ),
        (
            TileId(1427),
            ndarray::aview2(&[
                [1, 1, 1, 0, 1, 1, 0, 1, 0, 0],
                [0, 1, 0, 0, 1, 0, 1, 1, 0, 0],
                [0, 1, 0, 1, 1, 0, 1, 0, 0, 1],
                [1, 0, 1, 0, 1, 0, 1, 1, 0, 1],
                [0, 0, 0, 0, 1, 0, 0, 0, 1, 1],
                [0, 0, 0, 1, 1, 0, 0, 1, 1, 0],
                [0, 0, 0, 1, 0, 1, 1, 1, 1, 1],
                [0, 1, 0, 1, 1, 1, 1, 0, 1, 0],
                [0, 0, 1, 0, 0, 1, 1, 1, 0, 1],
                [0, 0, 1, 1, 0, 1, 0, 0, 1, 0],
            ]),
        ),
        (
            TileId(1489),
            ndarray::aview2(&[
                [1, 1, 0, 1, 0, 1, 0, 0, 0, 0],
                [0, 0, 1, 1, 0, 0, 0, 1, 0, 0],
                [0, 1, 1, 0, 0, 1, 1, 0, 0, 0],
                [0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
                [1, 1, 1, 1, 1, 0, 0, 0, 1, 0],
                [1, 0, 0, 1, 0, 1, 0, 1, 0, 1],
                [0, 0, 0, 1, 0, 1, 0, 1, 0, 0],
                [1, 1, 0, 1, 0, 0, 0, 1, 1, 0],
                [0, 0, 1, 1, 0, 1, 1, 0, 1, 1],
                [1, 1, 1, 0, 1, 1, 0, 1, 0, 0],
            ]),
        ),
        (
            TileId(2473),
            ndarray::aview2(&[
                [1, 0, 0, 0, 0, 1, 1, 1, 1, 0],
                [1, 0, 0, 1, 0, 1, 1, 0, 0, 0],
                [1, 0, 1, 1, 0, 0, 1, 0, 0, 0],
                [1, 1, 1, 1, 1, 1, 0, 1, 0, 1],
                [0, 1, 0, 0, 0, 1, 0, 1, 0, 1],
                [0, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [0, 1, 1, 1, 0, 1, 0, 0, 1, 0],
                [1, 1, 1, 1, 1, 1, 1, 1, 0, 1],
                [1, 1, 0, 0, 0, 1, 1, 0, 1, 0],
                [0, 0, 1, 1, 1, 0, 1, 0, 1, 0],
            ]),
        ),
        (
            TileId(2971),
            ndarray::aview2(&[
                [0, 0, 1, 0, 1, 0, 0, 0, 0, 1],
                [1, 0, 0, 0, 1, 1, 1, 0, 0, 0],
                [1, 0, 1, 0, 1, 1, 1, 0, 0, 0],
                [1, 1, 0, 1, 1, 0, 0, 1, 0, 0],
                [0, 1, 1, 1, 1, 1, 0, 0, 1, 1],
                [0, 1, 0, 0, 1, 1, 1, 1, 0, 1],
                [1, 0, 0, 1, 0, 1, 0, 0, 1, 0],
                [0, 0, 1, 1, 1, 1, 0, 1, 1, 1],
                [0, 0, 1, 0, 1, 0, 1, 1, 1, 0],
                [0, 0, 0, 1, 0, 1, 0, 1, 0, 1],
            ]),
        ),
        (
            TileId(2729),
            ndarray::aview2(&[
                [0, 0, 0, 1, 0, 1, 0, 1, 0, 1],
                [1, 1, 1, 1, 0, 1, 0, 0, 0, 0],
                [0, 0, 1, 0, 1, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 1, 0, 0, 1, 0, 1],
                [0, 1, 1, 0, 0, 1, 1, 0, 1, 0],
                [0, 1, 0, 1, 1, 1, 1, 0, 0, 0],
                [1, 1, 1, 1, 0, 1, 0, 1, 0, 0],
                [1, 1, 0, 1, 1, 1, 1, 0, 0, 0],
                [1, 1, 0, 0, 1, 0, 1, 1, 0, 0],
                [1, 0, 1, 1, 0, 0, 0, 1, 1, 0],
            ]),
        ),
        (
            TileId(3079),
            ndarray::aview2(&[
                [1, 0, 1, 0, 1, 1, 1, 1, 1, 0],
                [0, 1, 0, 0, 1, 1, 1, 1, 1, 1],
                [0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
                [1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
                [1, 1, 1, 1, 0, 1, 0, 0, 1, 0],
                [0, 1, 0, 0, 0, 1, 0, 1, 1, 0],
                [1, 0, 1, 1, 1, 1, 1, 0, 1, 1],
                [0, 0, 1, 0, 1, 1, 1, 0, 0, 0],
                [0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 1, 0, 1, 1, 1, 0, 0, 0],
            ]),
        ),
    ];

    const EXAMPLE_PUZZLE: ndarray::ArrayView2<u8> = ndarray::aview2(&[
        [
            0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1,
        ],
        [
            1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0,
        ],
        [
            1, 1, 0, 1, 1, 0, 1, 1, 1, 0, 1, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0,
        ],
        [
            1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1,
        ],
        [
            1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1,
        ],
        [
            0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1,
        ],
        [
            0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0, 0,
        ],
        [
            0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
        ],
        [
            1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0,
        ],
        [
            1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 0,
        ],
        [
            1, 1, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1, 1,
        ],
        [
            1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1,
        ],
        [
            1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0,
        ],
        [
            0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 1, 1,
        ],
        [
            0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0,
        ],
        [
            1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 0,
        ],
        [
            0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 0,
        ],
        [
            0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1,
        ],
        [
            0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1,
        ],
        [
            1, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0,
        ],
        [
            1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1,
        ],
        [
            1, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1,
        ],
        [
            0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1, 1, 0,
        ],
        [
            0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1,
        ],
    ]);

    const PURGED_EXAMPLE: &str = "\
.####...#####..#...###..
#####..#..#.#.####..#.#.
.#.#...#.###...#.##.O#..
#.O.##.OO#.#.OO.##.OOO##
..#O.#O#.O##O..O.#O##.##
...#.#..##.##...#..#..##
#.##.#..#.#..#..##.#.#..
.###.##.....#...###.#...
#.####.#.#....##.#..#.#.
##...#..#....#..#...####
..#.##...###..#.#####..#
....#.##.#.#####....#...
..##.##.###.....#.##..#.
#...#...###..####....##.
.#.##...#.##.#.#.###...#
#.###.#..####...##..#...
#.###...#.##...#.##O###.
.O##.#OO.###OO##..OOO##.
..O#.O..O..O.#O##O##.###
#.#..##.########..#..##.
#.#####..#.#...##..#....
#....##..#.#########..##
#...#.....#..##...###.##
#..###....##.#...##.##.#";
}
