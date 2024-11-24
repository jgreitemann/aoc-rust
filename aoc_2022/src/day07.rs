use aoc_companion::prelude::*;

use itertools::Itertools;
use thiserror::Error;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub struct Door {
    session: Vec<Command>,
}

impl ParseInput<'_> for Door {
    type Error = ParseError;

    fn parse(input: &str) -> Result<Self, Self::Error> {
        parse_session(input).map(|session| Self { session })
    }
}

impl Part1 for Door {
    type Output = usize;
    type Error = RuntimeError;

    fn part1(&self) -> Result<Self::Output, Self::Error> {
        Filesystem::from_session(&self.session).map(|fs| fs.total_size_of_small_directories())
    }
}

impl Part2 for Door {
    type Output = usize;
    type Error = RuntimeError;

    fn part2(&self) -> Result<Self::Output, Self::Error> {
        Filesystem::from_session(&self.session)
            .and_then(|fs| fs.size_of_directory_to_delete_to_make_space_for(30000000))
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error(transparent)]
    InvalidFileSize(#[from] std::num::ParseIntError),
    #[error("Prompt does not start with a dollar")]
    NoDollar,
    #[error("Unknown command: {0:?}")]
    UnknownCommand(String),
    #[error("Regex does not apply to directory listing entry: {0:?}")]
    RegexDoesNotMatch(String),
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("No such directory: {0}")]
    NoSuchDirectory(PathBuf),
    #[error("Not a directory: {0}")]
    NotADirectory(PathBuf),
    #[error("Inconsistent directory listing")]
    InconsistentDirectoryListing,
    #[error("Not enough space on device")]
    NotEnoughSpace,
}

#[derive(Debug, Clone, PartialEq)]
enum Command {
    Cd(PathBuf),
    Ls(Vec<DirListingEntry>),
}

#[derive(Debug, Clone, PartialEq)]
enum DirListingEntry {
    File(usize, PathBuf),
    Directory(PathBuf),
}

impl FromStr for Command {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(dirname) = s.strip_prefix("cd ") {
            Ok(Command::Cd(dirname.trim_end().into()))
        } else if let Some(output) = s.strip_prefix("ls") {
            Ok(Command::Ls(
                output.trim().lines().map(str::parse).try_collect()?,
            ))
        } else {
            Err(ParseError::UnknownCommand(s.to_owned()))
        }
    }
}

impl FromStr for DirListingEntry {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re =
            regex::Regex::new(r"^(dir (?P<dirname>.+)|(?P<size>\d+) (?P<filename>.+))$").unwrap();
        let capt = re
            .captures(s)
            .ok_or_else(|| ParseError::RegexDoesNotMatch(s.to_owned()))?;

        if let Some(dirname) = capt.name("dirname") {
            Ok(DirListingEntry::Directory(dirname.as_str().into()))
        } else if let (Some(size), Some(filename)) = (capt.name("size"), capt.name("filename")) {
            Ok(DirListingEntry::File(
                size.as_str().parse()?,
                filename.as_str().into(),
            ))
        } else {
            panic!("Logic error: Either the 'dirname' or both the 'size' and 'filename' captures have to exist if the regex matched.")
        }
    }
}

fn parse_session(input: &str) -> Result<Vec<Command>, ParseError> {
    if input.starts_with("$") {
        input.split("$ ").skip(1).map(str::parse).collect()
    } else {
        Err(ParseError::NoDollar)
    }
}

#[derive(Debug, Default, Clone)]
struct SessionState {
    cwd: PathBuf,
    fs: Filesystem,
}

impl SessionState {
    fn execute(mut self, cmd: &Command) -> Result<Self, RuntimeError> {
        match cmd {
            Command::Cd(dir) if dir == Path::new("..") => {
                let ok = self.cwd.pop();
                ok.then_some(self)
                    .ok_or(RuntimeError::NoSuchDirectory(PathBuf::from("/..")))
            }
            Command::Cd(dir) => {
                if let Some(parent) = self.fs.0.get(&self.cwd) {
                    self.cwd.push(dir);
                    match parent.iter().find(|node| {
                        matches!(node, FsNode::Directory(path) | FsNode::File(_, path) if path == &self.cwd)
                    }) {
                        Some(FsNode::Directory(_)) => Ok(self),
                        Some(FsNode::File(..)) => Err(RuntimeError::NotADirectory(self.cwd)),
                        None => Err(RuntimeError::NoSuchDirectory(self.cwd)),
                    }
                } else {
                    self.cwd.push(dir);
                    Ok(self)
                }
            }
            Command::Ls(contents) => {
                let prev = self.fs.0.insert(
                    self.cwd.clone(),
                    contents
                        .iter()
                        .map(|entry| FsNode::from_dir_entry(&self.cwd, entry))
                        .collect(),
                );

                match prev {
                    None => Ok(self),
                    Some(p) if p == self.fs.0[&self.cwd] => Ok(self),
                    Some(_) => Err(RuntimeError::InconsistentDirectoryListing),
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum FsNode {
    File(usize, PathBuf),
    Directory(PathBuf),
}

impl FsNode {
    fn from_dir_entry(cwd: &Path, entry: &DirListingEntry) -> Self {
        match entry {
            DirListingEntry::File(size, name) => FsNode::File(*size, cwd.join(name)),
            DirListingEntry::Directory(name) => FsNode::Directory(cwd.join(name)),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Filesystem(HashMap<PathBuf, Vec<FsNode>>);

impl Filesystem {
    fn from_session(session: &[Command]) -> Result<Self, RuntimeError> {
        session
            .iter()
            .try_fold(SessionState::default(), SessionState::execute)
            .map(|state| state.fs)
    }

    fn directory_size(&self, path: &Path) -> usize {
        self.0.get(path).map_or(0, |nodes| {
            nodes
                .iter()
                .map(|node| match node {
                    FsNode::File(size, _) => *size,
                    FsNode::Directory(path) => self.directory_size(path),
                })
                .sum()
        })
    }

    fn total_size_of_small_directories(&self) -> usize {
        self.0
            .keys()
            .map(|dir| self.directory_size(dir))
            .filter(|size| *size <= 100000)
            .sum()
    }

    fn size_of_directory_to_delete_to_free(&self, to_free: usize) -> Result<usize, RuntimeError> {
        self.0
            .keys()
            .map(|dir| self.directory_size(dir))
            .filter(|size| *size >= to_free)
            .min()
            .ok_or(RuntimeError::NotEnoughSpace)
    }

    fn total_size(&self) -> usize {
        self.directory_size(Path::new("/"))
    }

    fn size_of_directory_to_delete_to_make_space_for(
        &self,
        needed: usize,
    ) -> Result<usize, RuntimeError> {
        const CAPACITY: usize = 70000000;
        let space_left = CAPACITY
            .checked_sub(self.total_size())
            .ok_or(RuntimeError::NotEnoughSpace)?;
        self.size_of_directory_to_delete_to_free(needed - space_left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_matches::assert_matches;

    const EXAMPLE_INPUT: &str = r"$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k";

    fn example_session() -> Vec<Command> {
        use Command::*;
        use DirListingEntry::*;
        vec![
            Cd(PathBuf::from("/")),
            Ls(vec![
                Directory(PathBuf::from("a")),
                File(14848514, PathBuf::from("b.txt")),
                File(8504156, PathBuf::from("c.dat")),
                Directory(PathBuf::from("d")),
            ]),
            Cd(PathBuf::from("a")),
            Ls(vec![
                Directory(PathBuf::from("e")),
                File(29116, PathBuf::from("f")),
                File(2557, PathBuf::from("g")),
                File(62596, PathBuf::from("h.lst")),
            ]),
            Cd(PathBuf::from("e")),
            Ls(vec![File(584, PathBuf::from("i"))]),
            Cd(PathBuf::from("..")),
            Cd(PathBuf::from("..")),
            Cd(PathBuf::from("d")),
            Ls(vec![
                File(4060174, PathBuf::from("j")),
                File(8033020, PathBuf::from("d.log")),
                File(5626152, PathBuf::from("d.ext")),
                File(7214296, PathBuf::from("k")),
            ]),
        ]
    }

    fn example_filesystem() -> Filesystem {
        use FsNode::*;
        Filesystem(HashMap::from([
            (
                PathBuf::from("/"),
                vec![
                    Directory(PathBuf::from("/a")),
                    File(14848514, PathBuf::from("/b.txt")),
                    File(8504156, PathBuf::from("/c.dat")),
                    Directory(PathBuf::from("/d")),
                ],
            ),
            (
                PathBuf::from("/a"),
                vec![
                    Directory(PathBuf::from("/a/e")),
                    File(29116, PathBuf::from("/a/f")),
                    File(2557, PathBuf::from("/a/g")),
                    File(62596, PathBuf::from("/a/h.lst")),
                ],
            ),
            (
                PathBuf::from("/a/e"),
                vec![File(584, PathBuf::from("/a/e/i"))],
            ),
            (
                PathBuf::from("/d"),
                vec![
                    File(4060174, PathBuf::from("/d/j")),
                    File(8033020, PathBuf::from("/d/d.log")),
                    File(5626152, PathBuf::from("/d/d.ext")),
                    File(7214296, PathBuf::from("/d/k")),
                ],
            ),
        ]))
    }

    #[test]
    fn session_is_parsed() {
        assert_eq!(parse_session(EXAMPLE_INPUT).unwrap(), example_session());
    }

    #[test]
    fn unknown_commands_yield_error() {
        assert_matches!(parse_session("$ htop"), Err(ParseError::UnknownCommand(cmd)) if cmd == "htop");
    }

    #[test]
    fn unknown_directory_listing_entries_yield_error() {
        const SESSION_WITH_SYMLINK: &str = r"$ cd /
$ ls
dir foo
42000 a.txt
b.txt -> a.txt";
        assert_matches!(parse_session(SESSION_WITH_SYMLINK), Err(ParseError::RegexDoesNotMatch(entry))
                        if entry == "b.txt -> a.txt");
    }

    #[test]
    fn example_filesystem_is_reconstructed() {
        assert_eq!(
            Filesystem::from_session(&example_session()).unwrap(),
            example_filesystem()
        );
    }

    #[test]
    fn directories_are_assumed_to_exist_when_parent_has_not_been_listed() {
        const TEST_SESSION: &str = r"$ cd /
$ cd foo
$ ls
42000 bar.txt";
        assert_eq!(
            Filesystem::from_session(&parse_session(TEST_SESSION).unwrap()).unwrap(),
            Filesystem(HashMap::from([(
                PathBuf::from("/foo"),
                vec![FsNode::File(42000, PathBuf::from("/foo/bar.txt"))]
            )]))
        );
    }

    #[test]
    fn directories_have_to_exist_when_parent_has_been_listed() {
        const TEST_SESSION: &str = r"$ cd /
$ ls
dir foo
42000 bar";
        let fs = Filesystem::from_session(&parse_session(TEST_SESSION).unwrap()).unwrap();
        let state = SessionState {
            cwd: PathBuf::from("/"),
            fs: fs.clone(),
        };

        assert_matches!(
            state.clone().execute(&Command::Cd(PathBuf::from("foo"))),
            Ok(..)
        );

        assert_matches!(
            state.clone().execute(&Command::Cd(PathBuf::from("bar"))),
            Err(RuntimeError::NotADirectory(file)) if file == Path::new("/bar")
        );

        assert_matches!(
            state.clone().execute(&Command::Cd(PathBuf::from("baz"))),
            Err(RuntimeError::NoSuchDirectory(dir)) if dir == Path::new("/baz")
        );
    }

    #[test]
    fn total_size_of_small_directories() {
        assert_eq!(
            example_filesystem().total_size_of_small_directories(),
            95437
        );
    }

    #[test]
    fn size_of_directory_to_delete() {
        assert_eq!(
            example_filesystem()
                .size_of_directory_to_delete_to_make_space_for(30000000)
                .unwrap(),
            24933642
        );
    }

    #[test]
    fn trying_to_free_more_space_than_occupied_yields_error() {
        let fs = Filesystem(HashMap::from([(
            PathBuf::from("/"),
            vec![FsNode::File(42000, PathBuf::from("/foo"))],
        )]));

        assert_matches!(fs.size_of_directory_to_delete_to_free(40000), Ok(42000));
        assert_matches!(
            fs.size_of_directory_to_delete_to_free(45000),
            Err(RuntimeError::NotEnoughSpace)
        );
    }

    #[test]
    fn trying_to_exceed_capacity_of_device_yields_error() {
        assert_matches!(
            example_filesystem().size_of_directory_to_delete_to_make_space_for(80000000),
            Err(RuntimeError::NotEnoughSpace)
        );
    }
}
