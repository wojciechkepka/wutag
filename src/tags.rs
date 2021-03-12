//! Functions for manipulating tags on files.
use chrono::{offset::Utc, DateTime, NaiveDateTime};
use globwalk::DirEntry;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::convert::TryFrom;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::util;
use crate::xattr::{list_xattrs, remove_xattr, set_xattr};
use crate::{Error, Result, WUTAG_NAMESPACE};

#[derive(Debug, Eq)]
pub struct Tag {
    timestamp: DateTime<Utc>,
    name: String,
}

pub trait DirEntryExt {
    fn tag(&self, tag: &Tag) -> Result<()>;
    fn untag(&self, tag: &Tag) -> Result<()>;
    fn list_tags(&self) -> Result<Vec<Tag>>;
    fn list_tags_btree(&self) -> Result<BTreeSet<Tag>>;
    fn clear_tags(&self) -> Result<()>;
    fn has_tags(&self) -> Result<bool>;
    fn fmt_path(&self) -> String;
}

impl DirEntryExt for DirEntry {
    fn tag(&self, tag: &Tag) -> Result<()> {
        tag.save_to(self.path())
    }
    fn untag(&self, tag: &Tag) -> Result<()> {
        tag.remove_from(self.path())
    }
    fn list_tags(&self) -> Result<Vec<Tag>> {
        list_tags(self.path())
    }
    fn list_tags_btree(&self) -> Result<BTreeSet<Tag>> {
        list_tags_btree(self.path())
    }
    fn clear_tags(&self) -> Result<()> {
        clear_tags(self.path())
    }
    fn has_tags(&self) -> Result<bool> {
        has_tags(self.path())
    }
    fn fmt_path(&self) -> String {
        util::fmt_path(self.path())
    }
}

impl Tag {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Tag {
            timestamp: chrono::Utc::now(),
            name: name.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    fn xattr_name(&self) -> String {
        format!(
            "{}.{}.{}",
            WUTAG_NAMESPACE,
            self.timestamp.timestamp(),
            util::calculate_hash(&self.name)
        )
    }

    /// Tags the file at the given `path` with this tag. If the tag exists returns an error.
    pub fn save_to<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        for tag in list_tags(path.as_ref())? {
            if &tag == self {
                return Err(Error::TagExists);
            }
        }
        set_xattr(path, &self.xattr_name(), &self.name)
    }

    /// Removes this tag from the file at the given `path`. If the tag doesn't exists returns
    /// [Error::TagNotFound](wutag::Error::TagNotFound)
    pub fn remove_from<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        for (key, val) in list_xattrs(path.as_ref())? {
            // make sure to only remove attributes corresponding to this namespace
            if &val == &self.name && key.starts_with(WUTAG_NAMESPACE) {
                return remove_xattr(path, key);
            }
        }

        Err(Error::TagNotFound)
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        &self.name == &other.name
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl TryFrom<(String, String)> for Tag {
    type Error = Error;
    fn try_from(value: (String, String)) -> Result<Self> {
        let (key, value) = value;

        let mut elems = key.split('.');

        let ns = elems
            .next()
            .ok_or_else(|| Error::InvalidTagKey("missing namespace `user`".into()))?;
        if ns != "user" {
            return Err(Error::InvalidTagKey(format!(
                "invalid namespace `{}`, valid namespace is `user`",
                ns
            )));
        }
        let _wutag = elems
            .next()
            .ok_or_else(|| Error::InvalidTagKey("missing namespace `wutag`".into()))?;
        if ns != "user" {
            return Err(Error::InvalidTagKey(format!(
                "invalid namespace `{}`, valid namespace is `wutag`",
                ns
            )));
        }

        let timestamp = elems
            .next()
            .ok_or_else(|| Error::InvalidTagKey("missing timestamp".into()))?;

        let timestamp = NaiveDateTime::from_timestamp(
            timestamp
                .parse()
                .map_err(|e| Error::InvalidTagKey(format!("invalid timestamp - {}", e)))?,
            0,
        );

        Ok(Tag {
            timestamp: chrono::DateTime::<Utc>::from_utc(timestamp, Utc),
            name: value,
        })
    }
}

/// Lists tags of the file at the given `path`.
pub fn list_tags<P>(path: P) -> Result<Vec<Tag>>
where
    P: AsRef<Path>,
{
    list_xattrs(path).map(|attrs| {
        let mut tags = Vec::new();
        let mut it = attrs
            .into_iter()
            .filter(|(key, _)| key.starts_with(WUTAG_NAMESPACE))
            .map(Tag::try_from);

        while let Some(item) = it.next() {
            if let Ok(tag) = item {
                tags.push(tag);
            }
        }
        tags
    })
}

/// Lists tags of the file at the given `path` as a [BTreeSet](BTreeSet).
pub fn list_tags_btree<P>(path: P) -> Result<BTreeSet<Tag>>
where
    P: AsRef<Path>,
{
    list_xattrs(path).map(|attrs| {
        let mut tags = BTreeSet::new();
        let mut it = attrs
            .into_iter()
            .filter(|(key, _)| key.starts_with(WUTAG_NAMESPACE))
            .map(Tag::try_from);

        while let Some(item) = it.next() {
            if let Ok(tag) = item {
                tags.insert(tag);
            }
        }
        tags
    })
}

/// Clears all tags of the file at the given `path`.
pub fn clear_tags<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    for (key, _) in list_xattrs(path.as_ref())?
        .iter()
        .filter(|(key, _)| key.starts_with(WUTAG_NAMESPACE))
    {
        remove_xattr(path.as_ref(), key)?;
    }

    Ok(())
}

/// Searches for files with the specified tags in the location specified by `path`. If `recursive` is set to
/// `true` recursively follows all subdirectories.
///
/// Returns a list of paths of files that contain the provided set of tags.
pub fn search_files_with_tags<Ts, P>(tags: Ts, path: P, recursive: bool) -> Result<Vec<PathBuf>>
where
    Ts: IntoIterator<Item = String>,
    P: AsRef<Path>,
{
    let tags = tags.into_iter().map(Tag::new).collect::<BTreeSet<_>>();
    let mut files = Vec::new();

    let dir = path.as_ref().to_string_lossy().to_string();

    for entry in util::glob_walker(dir.as_str(), "**/*", recursive)? {
        if let Ok(entry) = entry {
            if let Ok(_tags) = list_tags_btree(entry.path()) {
                if !tags.is_subset(&_tags) {
                    // File doesn't have all tags
                    continue;
                }

                files.push(entry.path().to_path_buf());
            }
        }
    }

    Ok(files)
}

/// Checks whether the given path has any tags.
///
/// Returns an Error if the list of tags couldn't be aquired.
pub fn has_tags<P>(path: P) -> Result<bool>
where
    P: AsRef<Path>,
{
    list_tags(path).map(|tags| tags.len() > 0)
}
