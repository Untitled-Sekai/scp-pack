use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::path::Path;
use zip::ZipArchive;

use crate::error::Result;
use crate::utils::{validate_scp_file, prepare_output_dir};

pub struct PackExtractor;

impl PackExtractor {
    pub fn new() -> Self {
        Self
    }
    
    /// SCPファイルをpackディレクトリに展開
    pub fn extract(&self, scp_file: &Path, output_dir: &Path) -> Result<()> {
        validate_scp_file(scp_file)?;
        prepare_output_dir(output_dir)?;
        
        let file = File::open(scp_file)?;
        let mut archive = ZipArchive::new(file)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_path = output_dir.join(file.name());
            
            if file.is_dir() {
                create_dir_all(&file_path)?;
                println!("Created directory: {}", file.name());
            } else {
                if let Some(parent) = file_path.parent() {
                    create_dir_all(parent)?;
                }
                
                let mut output_file = File::create(&file_path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                output_file.write_all(&buffer)?;
                
                println!("Extracted: {}", file.name());
            }
        }
        
        println!("Successfully extracted to: {}", output_dir.display());
        Ok(())
    }
    
    /// SCPファイルの内容を一覧表示
    pub fn list_contents(&self, scp_file: &Path) -> Result<Vec<String>> {
        validate_scp_file(scp_file)?;
        
        let file = File::open(scp_file)?;
        let mut archive = ZipArchive::new(file)?;
        let mut contents = Vec::new();
        
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            contents.push(format!(
                "{} ({} bytes)", 
                file.name(), 
                file.size()
            ));
        }
        
        Ok(contents)
    }
    
    /// SCPファイル内の特定ファイルを読み込み
    pub fn read_file(&self, scp_file: &Path, file_path: &str) -> Result<Vec<u8>> {
        validate_scp_file(scp_file)?;
        
        let file = File::open(scp_file)?;
        let mut archive = ZipArchive::new(file)?;
        
        let mut target_file = archive.by_name(file_path)?;
        let mut buffer = Vec::new();
        target_file.read_to_end(&mut buffer)?;
        
        Ok(buffer)
    }
}