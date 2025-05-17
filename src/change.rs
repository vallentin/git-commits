use std::fmt;
use std::path::PathBuf;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl ChangeKind {
    #[inline]
    pub const fn letter(self) -> char {
        match self {
            Self::Added => 'A',
            Self::Modified => 'M',
            Self::Deleted => 'D',
            Self::Renamed => 'R',
        }
    }

    #[inline]
    pub const fn symbol(self) -> char {
        match self {
            Self::Added => '+',
            Self::Modified => '~',
            Self::Deleted => '-',
            Self::Renamed => '>',
        }
    }

    #[inline]
    pub const fn is_added(self) -> bool {
        matches!(self, Self::Added)
    }

    #[inline]
    pub const fn is_modified(self) -> bool {
        matches!(self, Self::Modified)
    }

    #[inline]
    pub const fn is_deleted(self) -> bool {
        matches!(self, Self::Deleted)
    }

    #[inline]
    pub const fn is_renamed(self) -> bool {
        matches!(self, Self::Renamed)
    }
}

impl fmt::Display for ChangeKind {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.letter().fmt(f)
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Change {
    Added(Added),
    Modified(Modified),
    Deleted(Deleted),
    Renamed(Renamed),
}

impl Change {
    #[inline]
    pub const fn kind(&self) -> ChangeKind {
        match self {
            Self::Added(_) => ChangeKind::Added,
            Self::Modified(_) => ChangeKind::Modified,
            Self::Deleted(_) => ChangeKind::Deleted,
            Self::Renamed(_) => ChangeKind::Renamed,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Added {
    /// The path of the added file.
    pub path: PathBuf,
    /// Total size in bytes.
    pub size: usize,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Modified {
    /// The path of the modified file.
    pub path: PathBuf,
    /// Total size in bytes.
    pub old_size: usize,
    /// Total size in bytes.
    pub new_size: usize,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Deleted {
    /// The path of the deleted file.
    pub path: PathBuf,
    /// Total size in bytes.
    pub size: usize,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Renamed {
    /// The path before the renaming.
    pub old_path: PathBuf,
    /// The path after the renaming.
    pub new_path: PathBuf,
    /// Total size in bytes.
    pub size: usize,
}

impl fmt::Display for Change {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Added(change) => change.fmt(f),
            Self::Modified(change) => change.fmt(f),
            Self::Deleted(change) => change.fmt(f),
            Self::Renamed(change) => change.fmt(f),
        }
    }
}

impl fmt::Display for Added {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} ({} bytes)",
            ChangeKind::Added,
            self.path.display(),
            self.size,
        )
    }
}

impl fmt::Display for Modified {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} ({} -> {} bytes)",
            ChangeKind::Modified,
            self.path.display(),
            self.old_size,
            self.new_size,
        )
    }
}

impl fmt::Display for Deleted {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} ({} bytes)",
            ChangeKind::Deleted,
            self.path.display(),
            self.size,
        )
    }
}

impl fmt::Display for Renamed {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} -> {} ({} bytes)",
            ChangeKind::Added,
            self.old_path.display(),
            self.new_path.display(),
            self.size,
        )
    }
}

impl From<Added> for Change {
    #[inline]
    fn from(change: Added) -> Self {
        Self::Added(change)
    }
}

impl From<Modified> for Change {
    #[inline]
    fn from(change: Modified) -> Self {
        Self::Modified(change)
    }
}

impl From<Deleted> for Change {
    #[inline]
    fn from(change: Deleted) -> Self {
        Self::Deleted(change)
    }
}

impl From<Renamed> for Change {
    #[inline]
    fn from(change: Renamed) -> Self {
        Self::Renamed(change)
    }
}
