use super::*;
use codespan_reporting::files::{Error, Files};
use std::{borrow::Cow, path::Path};

impl<'a> Codebase<'a> {
    /// Add a new source file to the codebase, providing the source code manually
    pub fn add_file(
        &mut self,
        path: impl Into<Cow<'a, Path>>,
        source: impl Into<Cow<'a, str>>,
    ) -> <Self as Files<'a>>::FileId {
        self.files.push(File::new(path, source));
        self.files.len() - 1
    }

    /// Add a new source file to the codebase, reading it from the file system
    pub fn read_file(
        &mut self,
        path: &Path,
    ) -> Result<<Self as Files<'a>>::FileId, std::io::Error> {
        Ok(self.add_file(path.to_owned(), std::fs::read_to_string(path)?))
    }

    /// Get the file corresponding to the given id.
    pub fn get_file(&self, file_id: usize) -> Result<&File<'a>, Error> {
        self.files.get(file_id).ok_or(Error::FileMissing)
    }
}

impl<'a> Files<'a> for Codebase<'a> {
    type FileId = usize;
    type Name = <codebase::File<'a> as Files<'a>>::Name;
    type Source = <codebase::File<'a> as Files<'a>>::Source;

    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, Error> {
        self.get_file(id)?.name(())
    }

    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, Error> {
        self.get_file(id)?.source(())
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, Error> {
        self.get_file(id)?.line_index((), byte_index)
    }

    fn line_range(
        &'a self,
        id: Self::FileId,
        line_index: usize,
    ) -> Result<std::ops::Range<usize>, Error> {
        self.get_file(id)?.line_range((), line_index)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A source file
pub struct File<'a> {
    path: Cow<'a, Path>,
    source: Cow<'a, str>,
    line_starts: Vec<usize>,
}

impl<'a> File<'a> {
    /// Create a new source file.
    pub fn new(path: impl Into<Cow<'a, Path>>, source: impl Into<Cow<'a, str>>) -> Self {
        let (path, source) = (path.into(), source.into());
        let line_starts = codespan_reporting::files::line_starts(source.as_ref()).collect();
        Self {
            path,
            source,
            line_starts,
        }
    }

    /// Return the starting byte index of the line with the specified line index.
    /// Convenience method that already generates errors if necessary.
    fn line_start(&self, line_index: usize) -> Result<usize, Error> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.line_starts.len()) {
            Ordering::Less => Ok(self
                .line_starts
                .get(line_index)
                .cloned()
                .expect("failed despite previous check")),
            Ordering::Equal => Ok(self.source.as_ref().len()),
            Ordering::Greater => Err(Error::LineTooLarge {
                given: line_index,
                max: self.line_starts.len() - 1,
            }),
        }
    }
}

impl<'a> std::fmt::Debug for File<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("File")
            .field("path", &self.path)
            .field("source", &self.source)
            .finish()
    }
}

impl<'a> Files<'a> for File<'a> {
    type FileId = ();
    type Name = FileName<'a>;
    type Source = &'a str;

    fn name(&'a self, (): ()) -> Result<Self::Name, Error> {
        Ok(FileName(self.path.as_ref()))
    }

    fn source(&'a self, (): ()) -> Result<Self::Source, Error> {
        Ok(self.source.as_ref())
    }

    fn line_index(&self, (): (), byte_index: usize) -> Result<usize, Error> {
        Ok(self
            .line_starts
            .binary_search(&byte_index)
            .unwrap_or_else(|next_line| next_line - 1))
    }

    fn line_range(&self, (): (), line_index: usize) -> Result<std::ops::Range<usize>, Error> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + 1)?;

        Ok(line_start..next_line_start)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A wrapped Path
pub struct FileName<'a>(&'a Path);

impl std::fmt::Display for FileName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl std::ops::Deref for FileName<'_> {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
