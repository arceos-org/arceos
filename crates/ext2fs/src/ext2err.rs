#[derive(Debug)]
/// Ext2Error
pub enum Ext2Error {
    /// A directory entry already exists
    AlreadyExists,
    /// A directory is not empty when delete
    DirectoryIsNotEmpty,
    /// An directory entity is not found
    NotFound,
    /// There is no enough storage space for write
    NotEnoughSpace,
    /// The entry has been deleted
    InvalidResource,
    /// The operation is only valid in file
    NotAFile,
    /// The operation is only valid in directory
    NotADir,
    /// Invalid inode number
    InvalidInodeId,
    /// Link to itself
    LinkToSelf,
    /// Link to directory
    LinkToDir,
    /// Path too long when doing symbolic link
    PathTooLong,
    /// Name too long when adding dentry in directory
    NameTooLong,
    /// Not a symbolic link
    NotSymlink,
    /// Invalid file/directory name
    InvalidName,
}

/// Ext2Result
pub type Ext2Result<T = ()> = Result<T, Ext2Error>;
