use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::path::Path;
use zip::ZipArchive;
use serde_json::Value;

use crate::error::Result;
use crate::utils::{validate_scp_file, prepare_output_dir};

pub struct PackExtractor;

impl PackExtractor {
    pub fn new() -> Self {
        Self
    }
    
    /// SCPファイルをpackディレクトリに展開（静的ファイル形式からpack形式に変換）
    pub fn extract(&self, scp_file: &Path, output_dir: &Path) -> Result<()> {
        validate_scp_file(scp_file)?;
        prepare_output_dir(output_dir)?;
        
        let file = File::open(scp_file)?;
        let mut archive = ZipArchive::new(file)?;
        
        // repositoryディレクトリを作成
        let repo_dir = output_dir.join("repository");
        create_dir_all(&repo_dir)?;
        
        // db.jsonの構造を準備
        let db = self.create_db_structure(&mut archive)?;
        
        // repositoryファイルをコピー
        self.extract_repository(&mut archive, &repo_dir)?;
        
        // db.jsonを書き込み
        let db_path = output_dir.join("db.json");
        let db_json = serde_json::to_string_pretty(&db)?;
        let mut db_file = File::create(&db_path)?;
        db_file.write_all(db_json.as_bytes())?;
        
        println!("Successfully converted to pack format: {}", output_dir.display());
        Ok(())
    }
    
    /// repositoryファイルを抽出
    fn extract_repository(&self, archive: &mut ZipArchive<File>, repo_dir: &Path) -> Result<()> {
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_name = file.name().to_string();
            
            // repositoryディレクトリ内のファイルのみを抽出
            if file_name.starts_with("static/sonolus/repository/") && !file.is_dir() {
                let hash = file_name.split('/').last().unwrap_or("");
                if !hash.is_empty() {
                    let output_path = repo_dir.join(hash);
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)?;
                    let mut output_file = File::create(&output_path)?;
                    output_file.write_all(&buffer)?;
                    println!("Copied repository file: {}", hash);
                }
            }
        }
        Ok(())
    }
    
    /// db.json構造を作成
    fn create_db_structure(&self, archive: &mut ZipArchive<File>) -> Result<Value> {
        let mut db = serde_json::json!({
            "info": {"title": {}},
            "posts": [],
            "playlists": [],
            "levels": [],
            "skins": [],
            "backgrounds": [],
            "effects": [],
            "particles": [],
            "engines": [],
            "replays": []
        });
        
        // 各カテゴリのlistファイルを処理
        let categories = vec![
            ("skins", "skins"),
            ("backgrounds", "backgrounds"),
            ("effects", "effects"),
            ("particles", "particles"),
            ("engines", "engines"),
            ("levels", "levels"),
            ("replays", "replays"),
            ("playlists", "playlists"),
            ("posts", "posts"),
        ];
        
        for (category, json_key) in categories {
            if let Ok(items) = self.read_list_from_archive(archive, category) {
                db[json_key] = items;
            }
        }
        
        // infoファイルを読み込み
        if let Ok(info) = self.read_info_from_archive(archive) {
            db["info"] = info;
        }
        
        Ok(db)
    }
    
    /// アーカイブからlistファイルを読み込み、pack形式に変換
    fn read_list_from_archive(&self, archive: &mut ZipArchive<File>, category: &str) -> Result<Value> {
        // まず個別アイテムページからdescriptionを収集
        let descriptions = self.collect_item_descriptions(archive, category)?;
        
        let list_path = format!("static/sonolus/{}/list", category);
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_name = file.name();
            
            if file_name == list_path {
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
                
                let list_data: Value = serde_json::from_str(&buffer)?;
                
                if let Some(items) = list_data["items"].as_array() {
                    let converted_items: Vec<Value> = items.iter()
                        .map(|item| {
                            let mut converted = self.convert_item_to_pack_format(item);
                            // descriptionを追加
                            if let Some(name) = item["name"].as_str() {
                                if let Some(description) = descriptions.get(name) {
                                    if !description.is_empty() {
                                        converted["description"] = serde_json::json!({"en": description});
                                    }
                                }
                            }
                            converted
                        })
                        .collect();
                    return Ok(Value::Array(converted_items));
                }
            }
        }
        
        Ok(Value::Array(vec![]))
    }
    
    /// カテゴリ内の全アイテムのdescriptionを収集
    fn collect_item_descriptions(&self, archive: &mut ZipArchive<File>, category: &str) -> Result<std::collections::HashMap<String, String>> {
        use std::collections::HashMap;
        
        let mut descriptions = HashMap::new();
        let prefix = format!("static/sonolus/{}/", category);
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_name = file.name().to_string();
            
            // カテゴリディレクトリ内のファイルで、"list"や"info"ではないものを処理
            if file_name.starts_with(&prefix) && !file.is_dir() {
                let item_name = file_name.strip_prefix(&prefix).unwrap_or("");
                
                if item_name != "list" && item_name != "info" && !item_name.is_empty() {
                    let mut buffer = String::new();
                    if file.read_to_string(&mut buffer).is_ok() {
                        if let Ok(item_data) = serde_json::from_str::<Value>(&buffer) {
                            if let Some(description) = item_data["description"].as_str() {
                                descriptions.insert(item_name.to_string(), description.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(descriptions)
    }
    
    /// アーカイブからinfoファイルを読み込み
    fn read_info_from_archive(&self, archive: &mut ZipArchive<File>) -> Result<Value> {
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name() == "static/sonolus/info" {
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
                let info_data: Value = serde_json::from_str(&buffer)?;
                return Ok(info_data);
            }
        }
        Ok(serde_json::json!({"title": {}}))
    }
    
    /// 静的ファイル形式のアイテムをpack形式に変換
    fn convert_item_to_pack_format(&self, item: &Value) -> Value {
        let mut converted = item.clone();
        
        // 文字列フィールドを {"en": "value"} 形式に変換
        let text_fields = vec!["title", "subtitle", "author", "description"];
        
        for field in text_fields {
            if let Some(value) = item.get(field) {
                if value.is_string() {
                    let text = value.as_str().unwrap_or("");
                    converted[field] = serde_json::json!({
                        "en": text
                    });
                }
            }
        }
        
        converted
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