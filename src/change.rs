use std::fmt;
use std::path::{Path, PathBuf};

macro_rules! change_impl {
    (
        $(($kind:ident, $as_kind:ident, $is_kind:ident)),*
        $(,)?
    ) => {
        impl Change {
            $(
                #[inline]
                pub fn $as_kind(&self) -> Option<&$kind> {
                    match &self {
                        Self::$kind(change) => Some(change),
                        _ => None,
                    }
                }

                #[inline]
                pub const fn $is_kind(&self) -> bool {
                    self.kind().$is_kind()
                }
            )*
        }
    };
}

macro_rules! change_kind_impl {
    (
        $kind:ident =>
        $($field:ident),*
        $(,)?
    ) => {
        impl $kind {
            #[inline]
            pub const fn kind(&self) -> ChangeKind {
                ChangeKind::$kind
            }

            $(
                change_kind_impl!(@expand $field);
            )*
        }
    };

    (@expand path) => {
        /// Returns the path of the file.
        #[inline]
        pub fn path(&self) -> &Path {
            &self.path.as_path()
        }

        /// Returns the path of the file.
        #[inline]
        pub fn into_path(self) -> PathBuf {
            self.path
        }
    };

    (@expand paths) => {
        /// Returns the path of the file, before the change.
        #[inline]
        pub fn old_path(&self) -> &Path {
            &self.old_path.as_path()
        }

        /// Returns the path of the file, after the change.
        #[doc(alias = "path")]
        #[inline]
        pub fn new_path(&self) -> &Path {
            &self.new_path.as_path()
        }

        /// Returns the `(old_path, new_path)`.
        #[inline]
        pub fn paths(&self) -> (&Path, &Path) {
            (self.old_path.as_path(), self.new_path.as_path())
        }

        /// Returns the `(old_path, new_path)`.
        #[inline]
        pub fn into_paths(self) -> (PathBuf, PathBuf) {
            (self.old_path, self.new_path)
        }

        #[doc(alias = "new_path")]
        #[inline]
        fn path(&self) -> &Path {
            &self.new_path.as_path()
        }

        #[inline]
        fn into_path(self) -> PathBuf {
            self.new_path
        }
    };

    (@expand size) => {
        /// Returns the total size in bytes of the file, before the change.
        #[inline]
        pub const fn size(&self) -> usize {
            self.size
        }
    };

    (@expand sizes) => {
        /// Returns the total size in bytes of the file, before the change.
        #[inline]
        pub const fn old_size(&self) -> usize {
            self.old_size
        }

        /// Returns the total size in bytes of the file, after the change.
        #[doc(alias = "size")]
        #[inline]
        pub const fn new_size(&self) -> usize {
            self.new_size
        }

        /// Returns the `(old_size, new_size)`.
        #[inline]
        pub fn sizes(&self) -> (usize, usize) {
            (self.old_size, self.new_size)
        }

        #[inline]
        const fn size(&self) -> usize {
            self.new_size
        }
    };
}

macro_rules! change {
    (
        $self:ident,
        $change:ident => $expr:expr
    ) => {
        match $self {
            Self::Added($change) => $expr,
            Self::Modified($change) => $expr,
            Self::Deleted($change) => $expr,
            Self::Renamed($change) => $expr,
        }
    };
}

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

    /// Returns the path of the file, after the change.
    #[doc(alias = "new_path")]
    #[inline]
    pub fn path(&self) -> &Path {
        change!(self, change => change.path())
    }

    /// Returns the path of the file, after the change.
    #[inline]
    pub fn into_path(self) -> PathBuf {
        change!(self, change => change.into_path())
    }

    /// Returns the path of the file, before the change.
    ///
    /// Only <code>[Change]::[Renamed]</code>
    /// has an [`old_path`](Renamed::old_path).
    #[inline]
    pub fn old_path(&self) -> Option<&Path> {
        match self {
            Self::Renamed(change) => Some(change.old_path()),
            _ => None,
        }
    }

    /// Returns the `(old_path, path)`.
    ///
    /// Only <code>[Change]::[Renamed]</code>
    /// has an [`old_path`](Renamed::old_path).
    #[inline]
    pub fn paths(&self) -> (Option<&Path>, &Path) {
        match self {
            Self::Added(change) => (None, change.path()),
            Self::Modified(change) => (None, change.path()),
            Self::Deleted(change) => (None, change.path()),
            Self::Renamed(change) => {
                let (old_path, new_path) = change.paths();
                (Some(old_path), new_path)
            }
        }
    }

    /// Returns the `(old_path, path)`.
    ///
    /// Only <code>[Change]::[Renamed]</code>
    /// has an [`old_path`](Renamed::old_path).
    #[inline]
    pub fn into_paths(self) -> (Option<PathBuf>, PathBuf) {
        match self {
            Self::Added(change) => (None, change.into_path()),
            Self::Modified(change) => (None, change.into_path()),
            Self::Deleted(change) => (None, change.into_path()),
            Self::Renamed(change) => {
                let (old_path, new_path) = change.into_paths();
                (Some(old_path), new_path)
            }
        }
    }

    /// Returns the total size in bytes of the file, after the change.
    #[doc(alias = "new_size")]
    #[inline]
    pub const fn size(&self) -> usize {
        change!(self, change => change.size())
    }

    /// Returns the total size in bytes of the file, before the change.
    ///
    /// Only <code>[Change]::[Modified]</code>
    /// has an [`old_size`](Modified::old_size).
    #[inline]
    pub const fn old_size(&self) -> Option<usize> {
        match self {
            Self::Modified(change) => Some(change.old_size()),
            _ => None,
        }
    }

    /// Returns the `(old_size, size)`.
    ///
    /// Only <code>[Change]::[Modified]</code>
    /// has an [`old_size`](Modified::old_size).
    #[inline]
    pub const fn sizes(&self) -> (Option<usize>, usize) {
        match self {
            Self::Added(change) => (None, change.size()),
            Self::Modified(change) => (Some(change.old_size()), change.size()),
            Self::Deleted(change) => (None, change.size()),
            Self::Renamed(change) => (None, change.size()),
        }
    }
}

change_impl!(
    (Added, as_added, is_added),
    (Modified, as_modified, is_modified),
    (Deleted, as_deleted, is_deleted),
    (Renamed, as_renamed, is_renamed),
);

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Added {
    /// The path of the added file.
    pub(crate) path: PathBuf,
    /// Total size in bytes.
    pub(crate) size: usize,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Modified {
    /// The path of the modified file.
    pub(crate) path: PathBuf,
    /// Total size in bytes.
    pub(crate) old_size: usize,
    /// Total size in bytes.
    pub(crate) new_size: usize,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Deleted {
    /// The path of the deleted file.
    pub(crate) path: PathBuf,
    /// Total size in bytes.
    pub(crate) size: usize,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Renamed {
    /// The path before the renaming.
    pub(crate) old_path: PathBuf,
    /// The path after the renaming.
    pub(crate) new_path: PathBuf,
    /// Total size in bytes.
    pub(crate) size: usize,
}

change_kind_impl!(Added => path, size);
change_kind_impl!(Modified => path, sizes);
change_kind_impl!(Deleted => path, size);
change_kind_impl!(Renamed => paths, size);

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
