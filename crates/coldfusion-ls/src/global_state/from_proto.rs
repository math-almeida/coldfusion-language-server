use lsp_types::Url;
use virtual_fs::{AbsPathBuf, VirtualFsPath};
pub(crate) fn abs_path(url: &Url) -> anyhow::Result<AbsPathBuf> {
    let path = url
        .to_file_path()
        .map_err(|()| anyhow::format_err!("url is not a file: {}", url))?;
    Ok(AbsPathBuf::try_from(path).unwrap())
}

pub(crate) fn vfs_path(url: &Url) -> anyhow::Result<VirtualFsPath> {
    abs_path(url).map(|it| VirtualFsPath::from(it))
}
