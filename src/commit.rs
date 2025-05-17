use std::borrow::Cow;

use git2::Repository;

use super::Changes;
use super::GitError;

pub struct Commit<'repo> {
    pub(crate) repo: &'repo Repository,
    pub(crate) commit: git2::Commit<'repo>,
}

impl<'repo> Commit<'repo> {
    #[inline]
    pub(crate) const fn new(repo: &'repo Repository, commit: git2::Commit<'repo>) -> Self {
        Self { repo, commit }
    }

    #[inline]
    pub fn sha(&self) -> String {
        self.commit.id().to_string()
    }

    #[inline]
    pub fn message_bytes(&self) -> &[u8] {
        self.commit.message_bytes()
    }

    #[inline]
    pub fn message(&self) -> Option<&str> {
        self.commit.message()
    }

    #[inline]
    pub fn message_lossy(&self) -> String {
        let msg = self.message_bytes();
        String::from_utf8_lossy(msg).into_owned()
    }

    #[inline]
    pub fn author(&self) -> Signature<'_> {
        Signature {
            signature: self.commit.author(),
        }
    }

    #[inline]
    pub fn committer(&self) -> Signature<'_> {
        Signature {
            signature: self.commit.committer(),
        }
    }

    #[inline]
    pub fn changes(&self) -> Result<Changes<'repo, '_>, GitError> {
        Changes::from_commit(self)
    }
}

pub struct Signature<'a> {
    signature: git2::Signature<'a>,
}

impl Signature<'_> {
    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.signature.name()
    }

    #[inline]
    pub fn name_bytes(&self) -> &[u8] {
        self.signature.name_bytes()
    }

    #[inline]
    pub fn name_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.name_bytes())
    }

    #[inline]
    pub fn email(&self) -> Option<&str> {
        self.signature.email()
    }

    #[inline]
    pub fn email_bytes(&self) -> &[u8] {
        self.signature.email_bytes()
    }

    #[inline]
    pub fn email_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.email_bytes())
    }

    #[inline]
    pub fn when(&self) -> (i64, i32) {
        let time = self.signature.when();
        (time.seconds(), time.offset_minutes())
    }
}
