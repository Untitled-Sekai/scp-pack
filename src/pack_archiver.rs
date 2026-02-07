use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::{ZipWriter, CompressionMethod};
use zip::write::SimpleFileOptions;
use serde_json::Value;

use crate::error::Result;
use crate::utils::{validate_pack_dir, prepare_output_dir};

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
    
    /// packディレクトリをscpファイルにアーカイブ（pack形式から静的ファイル形式に変換）
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
        
        // db.jsonを読み込み
        let db_path = pack_dir.join("db.json");
        let db_content = std::fs::read_to_string(&db_path)?;
        let db: Value = serde_json::from_str(&db_content)?;
        
        // 静的ファイル形式に変換してアーカイブに追加
        self.add_static_files(&mut zip, &db, pack_dir, &options)?;
        
        zip.finish()?;
        println!("Successfully created SCP file: {}", output_path.display());
        
        Ok(())
    }
    
    /// 静的ファイル形式のファイルをアーカイブに追加
    fn add_static_files(
        &self,
        zip: &mut ZipWriter<File>,
        db: &Value,
        pack_dir: &Path,
        options: &SimpleFileOptions,
    ) -> Result<()> {
        // infoファイルを追加
        self.add_info_file(zip, &db["info"], options)?;
        
        // packageファイルを追加
        self.add_package_file(zip, options)?;
        
        // repositoryファイルをコピー
        self.add_repository_files(zip, pack_dir, options)?;
        
        // 各カテゴリを処理
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
        
        for (json_key, category) in categories {
            if let Some(items) = db[json_key].as_array() {
                self.add_category_files(zip, category, items, options)?;
            }
        }
        
        Ok(())
    }
    
    /// infoファイルを追加
    fn add_info_file(
        &self,
        zip: &mut ZipWriter<File>,
        info: &Value,
        options: &SimpleFileOptions,
    ) -> Result<()> {
        let path = "static/sonolus/info";
        let content = serde_json::to_string(info)?;
        
        zip.start_file(path, *options)?;
        zip.write_all(content.as_bytes())?;
        println!("Added: {}", path);
        
        Ok(())
    }
    
    /// packageファイルを追加
    fn add_package_file(
        &self,
        zip: &mut ZipWriter<File>,
        options: &SimpleFileOptions,
    ) -> Result<()> {
        let path = "static/sonolus/package";
        let content = "{}";
        
        zip.start_file(path, *options)?;
        zip.write_all(content.as_bytes())?;
        println!("Added: {}", path);
        
        Ok(())
    }
    
    /// repositoryファイルを追加
    fn add_repository_files(
        &self,
        zip: &mut ZipWriter<File>,
        pack_dir: &Path,
        options: &SimpleFileOptions,
    ) -> Result<()> {
        let repo_dir = pack_dir.join("repository");
        
        if repo_dir.exists() {
            for entry in WalkDir::new(&repo_dir) {
                let entry = entry.map_err(|e| std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read repository entry: {}", e)
                ))?;
                
                let path = entry.path();
                
                if path.is_file() {
                    let file_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .ok_or_else(|| std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid file name"
                        ))?;
                    
                    let zip_path = format!("static/sonolus/repository/{}", file_name);
                    
                    let mut file = File::open(path)?;
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)?;
                    
                    zip.start_file(&zip_path, *options)?;
                    zip.write_all(&buffer)?;
                    
                    println!("Added: {}", zip_path);
                }
            }
        }
        
        Ok(())
    }
    
    /// カテゴリのファイルを追加
    fn add_category_files(
        &self,
        zip: &mut ZipWriter<File>,
        category: &str,
        items: &[Value],
        options: &SimpleFileOptions,
    ) -> Result<()> {
        // listファイルを作成
        let list_items: Vec<Value> = items.iter()
            .map(|item| self.convert_item_to_static_format(item))
            .collect();
        
        let list_data = serde_json::json!({
            "pageCount": 1,
            "items": list_items
        });
        
        let list_path = format!("static/sonolus/{}/list", category);
        let list_content = serde_json::to_string(&list_data)?;
        
        zip.start_file(&list_path, *options)?;
        zip.write_all(list_content.as_bytes())?;
        println!("Added: {}", list_path);
        
        // 各アイテムの個別ページを作成
        for item in items {
            if let Some(name) = item["name"].as_str() {
                self.add_item_file(zip, category, name, item, options)?;
            }
        }
        
        // infoファイルを作成（カテゴリ用）
        self.add_category_info_file(zip, category, options)?;
        
        Ok(())
    }
    
    /// アイテムの個別ページを追加
    fn add_item_file(
        &self,
        zip: &mut ZipWriter<File>,
        category: &str,
        name: &str,
        item: &Value,
        options: &SimpleFileOptions,
    ) -> Result<()> {
        let item_path = format!("static/sonolus/{}/{}", category, name);
        
        // descriptionを抽出
        let description = item.get("description")
            .and_then(|d| d.get("en"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // itemTypeを単数形に変換
        let item_type = match category {
            "skins" => "skin",
            "backgrounds" => "background",
            "effects" => "effect",
            "particles" => "particle",
            "engines" => "engine",
            "levels" => "level",
            "replays" => "replay",
            "playlists" => "playlist",
            "posts" => "post",
            _ => category,
        };
        
        let item_data = serde_json::json!({
            "item": self.convert_item_to_static_format(item),
            "description": description,
            "actions": [],
            "hasCommunity": false,
            "leaderboards": [],
            "sections": [
                {
                    "title": "#RECOMMENDED",
                    "icon": "star",
                    "itemType": item_type,
                    "items": []
                }
            ]
        });
        
        let item_content = serde_json::to_string(&item_data)?;
        
        zip.start_file(&item_path, *options)?;
        zip.write_all(item_content.as_bytes())?;
        println!("Added: {}", item_path);
        
        Ok(())
    }
    
    /// カテゴリのinfoファイルを追加
    fn add_category_info_file(
        &self,
        zip: &mut ZipWriter<File>,
        category: &str,
        options: &SimpleFileOptions,
    ) -> Result<()> {
        let info_path = format!("static/sonolus/{}/info", category);
        
        let info_data = serde_json::json!({
            "search": {
                "options": []
            }
        });
        
        let info_content = serde_json::to_string(&info_data)?;
        
        zip.start_file(&info_path, *options)?;
        zip.write_all(info_content.as_bytes())?;
        println!("Added: {}", info_path);
        
        Ok(())
    }
    
    /// pack形式のアイテムを静的ファイル形式に変換
    fn convert_item_to_static_format(&self, item: &Value) -> Value {
        let mut converted = item.clone();
        
        // {"en": "value"}形式を"value"形式に変換
        let text_fields = vec!["title", "subtitle", "author"];
        
        for field in text_fields {
            if let Some(value) = item.get(field) {
                if let Some(en_value) = value.get("en") {
                    if let Some(text) = en_value.as_str() {
                        converted[field] = Value::String(text.to_string());
                    }
                }
            }
        }
        
        // descriptionは個別ページにのみ含まれるため、listからは削除
        if converted.is_object() {
            converted.as_object_mut().unwrap().remove("description");
        }
        
        converted
    }
}