use std::path::Path;

use crate::error::Result;
use crate::pack_archiver::PackArchiver;
use crate::pack_extractor::PackExtractor;

pub struct Converter {
    archiver: PackArchiver,
    extractor: PackExtractor,
}

impl Converter {
    pub fn new() -> Self {
        Self {
            archiver: PackArchiver::new(),
            extractor: PackExtractor::new(),
        }
    }
    
    pub fn with_compression_level(mut self, level: i64) -> Self {
        self.archiver = self.archiver.with_compression_level(level);
        self
    }
    
    /// packディレクトリをSCPファイルに変換
    pub fn pack_to_scp(&self, pack_dir: &Path, scp_file: &Path) -> Result<()> {
        println!("Converting pack directory to SCP file...");
        println!("Input: {}", pack_dir.display());
        println!("Output: {}", scp_file.display());
        
        self.archiver.archive(pack_dir, scp_file)?;
        Ok(())
    }
    
    /// SCPファイルをpackディレクトリに変換
    pub fn scp_to_pack(&self, scp_file: &Path, pack_dir: &Path) -> Result<()> {
        println!("Converting SCP file to pack directory...");
        println!("Input: {}", scp_file.display());
        println!("Output: {}", pack_dir.display());
        
        self.extractor.extract(scp_file, pack_dir)?;
        Ok(())
    }
    
    /// SCPファイルの内容を表示
    pub fn list_scp_contents(&self, scp_file: &Path) -> Result<()> {
        println!("Contents of {}:", scp_file.display());
        let contents = self.extractor.list_contents(scp_file)?;
        
        for (index, content) in contents.iter().enumerate() {
            println!("  {}: {}", index + 1, content);
        }
        
        Ok(())
    }
    
    /// SCPファイル内の特定ファイルを表示
    pub fn show_file(&self, scp_file: &Path, file_path: &str) -> Result<()> {
        let content = self.extractor.read_file(scp_file, file_path)?;
        
        if let Ok(text) = String::from_utf8(content.clone()) {
            println!("Content of {} in {}:", file_path, scp_file.display());
            println!("{}", text);
        } else {
            println!("File {} is binary ({} bytes)", file_path, content.len());
        }
        
        Ok(())
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}