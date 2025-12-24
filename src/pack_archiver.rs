use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::{ZipWriter, CompressionMethod};
use zip::write::SimpleFileOptions;

use crate::error::Result;
use crate::utils::{normalize_path, validate_pack_dir, prepare_output_dir};

pub struct PackArchiver {
    compression_level: i64,
}

impl PackArchiver {
    pub fn new() -> Self {
        Self {
            compression_level: 6,
        }
    }
    
    pub fn with_compression_level(mut self, level: i64) -> Self {
        self.compression_level = level;
        self
    }
    
    /// packディレクトリをscpファイルにアーカイブ
    pub fn archive(&self, pack_dir: &Path, output_path: &Path) -> Result<()> {
        validate_pack_dir(pack_dir)?;
        
        if let Some(parent) = output_path.parent() {
            prepare_output_dir(parent)?;
        }
        
        let file = File::create(output_path)?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(self.compression_level));
        
        self.add_directory_to_zip(&mut zip, pack_dir, pack_dir, &options)?;
        
        zip.finish()?;
        println!("Successfully created SCP file: {}", output_path.display());
        
        Ok(())
    }
    
    fn add_directory_to_zip(
        &self,
        zip: &mut ZipWriter<File>,
        dir_path: &Path,
        base_path: &Path,
        options: &SimpleFileOptions,
    ) -> Result<()> {
        for entry in WalkDir::new(dir_path) {
            let entry = entry.map_err(|e| std::io::Error::new(
                std::io::ErrorKind::Other, 
                format!("Failed to read directory entry: {}", e)
            ))?;
            
            let path = entry.path();
            
            if path.is_file() {
                let relative_path = normalize_path(path, base_path)?;
                
                let mut file = File::open(path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                
                zip.start_file(&relative_path, *options)?;
                zip.write_all(&buffer)?;
                
                println!("Added: {}", relative_path);
            }
        }
        
        Ok(())
    }
}