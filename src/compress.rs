#[cfg(feature = "zip")]
extern crate zip;

#[cfg(feature = "tgz")]
extern crate flate2;
#[cfg(feature = "tar")]
extern crate tar;

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path;

use failure;

use format::Format;

#[cfg(feature = "zip")]
fn compress_zip(root: &path::Path, output: &path::Path) -> Result<(), failure::Error> {
    use globwalk;

    let file = File::create(output)?;
    let mut zip = zip::ZipWriter::new(file);
    let mut buffer = Vec::new();
    for entry in globwalk::GlobWalkerBuilder::new(root, "*").build()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path.strip_prefix(root)
                .expect("root is still prefix")
                .to_str()
                .ok_or_else(|| format_err!("Invalid character in path: {:?}", path))?;
            // TODO(epage): Read permissions from disc
            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o766);
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        }
    }
    zip.finish()?;
    Ok(())
}

#[cfg(not(feature = "zip"))]
fn compress_zip(_root: &path::Path, _output: &path::Path) -> Result<(), failure::Error> {
    bail!("zip is not supported");
}

#[cfg(feature = "tar")]
fn compress_tar(root: &path::Path, output: &path::Path) -> Result<(), failure::Error> {
    let file = File::create(output)?;
    let mut archive = tar::Builder::new(file);
    archive.append_dir_all(".", root)?;
    archive.finish()?;
    Ok(())
}

#[cfg(not(feature = "tar"))]
fn compress_tar(root: &path::Path, output: &path::Path) -> Result<(), failure::Error> {
    bail!("tar is not supported");
}

#[cfg(feature = "tgz")]
fn compress_tgz(root: &path::Path, output: &path::Path) -> Result<(), failure::Error> {
    let buffer = Vec::new();
    let mut archive = tar::Builder::new(buffer);
    archive.append_dir_all(".", root)?;
    let buffer = archive.into_inner()?;
    let file = File::create(output)?;
    let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    encoder.write_all(&buffer)?;
    encoder.finish()?;
    Ok(())
}

#[cfg(not(feature = "tgz"))]
fn compress_tgz(_root: &path::Path, _output: &path::Path) -> Result<(), failure::Error> {
    bail!("tgz is not supported");
}

pub fn compress(
    root: &path::Path,
    output: &path::Path,
    format: Format,
) -> Result<(), failure::Error> {
    match format {
        Format::Tar => compress_tar(root, output),
        Format::Tgz => compress_tgz(root, output),
        Format::Zip => compress_zip(root, output),
    }
}
