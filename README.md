# Role Panel Bot

ボタンやセレクトメニューでロール選択パネルを管理するDiscord Bot

## 機能

- ボタンまたはセレクトメニューでインタラクティブなロールパネルを作成
- 複数ロール選択のサポート
- サーバー固有の監査ログ
- ヘルスチェック付きKubernetes対応

## 要件

- Rust 1.75.0以降
- PostgreSQL 14以降
- Discord Bot Token

## 設定

| 変数 | 説明 | 必須 | デフォルト |
|------|------|------|-----------|
| DISCORD_TOKEN | Botトークン | Yes | - |
| DATABASE_URL | PostgreSQL接続文字列 | Yes | - |
| RUST_LOG | ログレベル | No | info |
| HEALTH_PORT | ヘルスサーバーポート | No | 8080 |
| DATABASE_MAX_CONNECTIONS | コネクションプールサイズ | No | 5 |

## 開発

```bash
# 環境変数の読み込み
cp .env.example .env
# .envに値を設定

# マイグレーション実行
sqlx migrate run

# Bot起動
cargo run
```

## ビルド

```bash
cargo build --release
```

## Docker

```bash
docker build -t role-panel-bot .
docker run -e DISCORD_TOKEN=... -e DATABASE_URL=... role-panel-bot
```

## Kubernetes

### 前提条件

- kubectl設定済み
- Kubernetesクラスターへのアクセス

### デプロイ手順

1. Secretの設定

```bash
# Discord TokenとPostgreSQLパスワードを設定
kubectl create secret generic role-panel-bot-secrets \
  --from-literal=discord-token=YOUR_DISCORD_TOKEN_HERE \
  --from-literal=database-url=postgres://role_panel:YOUR_PASSWORD@postgres:5432/role_panel

kubectl create secret generic postgres-secrets \
  --from-literal=username=role_panel \
  --from-literal=password=YOUR_PASSWORD
```

2. PostgreSQLのデプロイ

```bash
kubectl apply -f deploy/kubernetes/postgres.yaml
```

3. Botのデプロイ

```bash
kubectl apply -f deploy/kubernetes/deployment.yaml
kubectl apply -f deploy/kubernetes/service.yaml
```

4. デプロイ確認

```bash
# Pod状態確認
kubectl get pods

# ログ確認
kubectl logs -f deployment/role-panel-bot

# ヘルスチェック確認
kubectl get pods -l app=role-panel-bot
```

### マイグレーション実行

```bash
# PostgreSQL Podに接続
kubectl exec -it postgres-0 -- psql -U role_panel -d role_panel

# マイグレーションファイルを手動実行、または
# 初回起動時にBot側で自動実行される場合はスキップ
```

### アンデプロイ

```bash
kubectl delete -f deploy/kubernetes/service.yaml
kubectl delete -f deploy/kubernetes/deployment.yaml
kubectl delete -f deploy/kubernetes/postgres.yaml
kubectl delete secret role-panel-bot-secrets postgres-secrets
```

## コマンド

| コマンド | 説明 |
|---------|------|
| /panel create | 新しいロールパネルを作成 |
| /panel list | 全パネルをリスト表示 |
| /panel edit | 既存パネルを編集 |
| /config audit-channel | 監査ログチャンネルを設定 |
| /config show | 現在の設定を表示 |

## ライセンス

MIT
