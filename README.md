# scp-pack

SCPファイルとpackディレクトリ間の変換を行うツールです。

## 概要

- `.scp`ファイルはZIP圧縮されたアーカイブファイルです
- `pack`ディレクトリは展開された状態のファイル構造です
- 双方向の変換をサポートしています

## インストール

```bash
cargo build --release
```

## 使用方法

### Pack → SCP変換

packディレクトリをSCPファイルに変換します：

```bash
cargo run -- pack -i "example/pack" -o "output.scp"
```

### SCP → Pack変換

SCPファイルをpackディレクトリに展開します：

```bash
cargo run -- unpack -i "input.scp" -o "output_dir"
```

### SCPファイルの内容確認

SCPファイルに含まれるファイル一覧を表示します：

```bash
cargo run -- list -f "input.scp"
```

### 特定ファイルの内容表示

SCPファイル内の特定ファイルを表示します：

```bash
cargo run -- show -s "input.scp" -f "db.json"
```

## オプション

### 圧縮レベル

`-c`または`--compression`オプションで圧縮レベル（0-9）を指定できます：

```bash
cargo run -- pack -i "pack" -o "output.scp" -c 9
```

## プロジェクト構造

```
src/
├── main.rs           # CLIエントリーポイント
├── lib.rs            # ライブラリルート
├── converter.rs      # メイン変換ロジック
├── pack_archiver.rs  # Pack → SCP変換
├── pack_extractor.rs # SCP → Pack変換
├── error.rs          # エラー型定義
└── utils.rs          # ユーティリティ関数
```

## 機能

- **Pack → SCP**: packディレクトリを圧縮してSCPファイルを作成
- **SCP → Pack**: SCPファイルを展開してpackディレクトリを作成
- **内容確認**: SCPファイルの内容一覧表示
- **ファイル表示**: SCPファイル内の特定ファイル内容表示
- **設定可能な圧縮レベル**: 0（無圧縮）から9（最高圧縮）まで選択可能
- **エラーハンドリング**: 詳細なエラーメッセージとバリデーション

## 依存関係

- `zip`: ZIP圧縮・展開
- `clap`: コマンドライン引数解析
- `anyhow`: エラーハンドリング
- `walkdir`: ディレクトリトラバーサル
- `serde`, `serde_json`: JSON処理