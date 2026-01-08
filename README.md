# Role Panel Bot

ボタンやセレクトメニューでロール選択パネルを管理するDiscord Bot

## 機能

- ボタンまたはセレクトメニューでインタラクティブなロールパネルを作成
- 複数ロール選択のサポート
- サーバー固有の監査ログ
- ヘルスチェック付きKubernetes対応

## 要件

- Rust 1.85.0以降（edition2024対応）
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

## Kubernetes (Helm)

### 前提条件

- kubectl設定済み
- Helm 3.x インストール済み
- Kubernetesクラスターへのアクセス
- Docker（イメージビルド用）

### デプロイ手順

#### 1. サーバー上でDockerイメージをローカルビルド

コンテナレジストリを使用せず、デプロイ先のサーバー上で直接Dockerイメージをビルドします。

```bash
git clone https://github.com/Aqua-218/RolePanel-Bot.git
cd RolePanel-Bot

docker build -t role-panel-bot:latest .

docker images | grep role-panel-bot
```

> **注意**: ビルドには数分かかります。vendorディレクトリが含まれているためオフラインビルドが可能です。

#### 2. values.yamlの設定

```bash
# values.yamlを編集
vi values.yaml
```

```yaml
# 必須設定
image:
  repository: role-panel-bot
  pullPolicy: Never  # ローカルイメージを使用（重要！）
  tag: "latest"

discord:
  token: "YOUR_DISCORD_TOKEN_HERE"

postgresql:
  host: "postgres-service"  # クラスター内のPostgreSQLサービス名
  port: 5432
  database: "role_panel"
  username: "role_panel"
  auth:
    password: "YOUR_POSTGRES_PASSWORD"

# オプション設定
errorWebhook:
  enabled: true
  url: "https://discord.com/api/webhooks/xxx/yyy"

botInfo:
  name: "Role Panel Bot"
  description: "ロール選択パネルを管理するBot"
  developerId: "YOUR_DISCORD_USER_ID"
  githubUrl: "https://github.com/Aqua-218/RolePanel-Bot"
```

> **重要**: `image.pullPolicy: Never` を設定することで、Kubernetesはレジストリからpullせずローカルイメージを使用します。

#### 3. Helmでデプロイ

```bash
# インストール
helm install role-panel-bot ./helm/role-panel-bot -f values.yaml

# 既存Secretを使用する場合
helm install role-panel-bot ./helm/role-panel-bot -f values.yaml \
  --set discord.existingSecret=my-discord-secret
```

#### 4. デプロイ確認

```bash
# Pod状態確認
kubectl get pods -l app.kubernetes.io/name=role-panel-bot

# ログ確認
kubectl logs -f deployment/role-panel-bot

# ヘルスチェック確認
helm status role-panel-bot
```

#### トラブルシューティング

**ImagePullBackOff エラーの場合:**
```bash
# イメージがローカルに存在するか確認
docker images | grep role-panel-bot

# values.yamlで pullPolicy: Never が設定されているか確認
grep -A2 "image:" values.yaml
```

**Podが起動しない場合:**
```bash
# Pod詳細確認
kubectl describe pod -l app.kubernetes.io/name=role-panel-bot

# イベント確認
kubectl get events --sort-by='.lastTimestamp'
```

### Webhook/Ingress設定

```yaml
# my-values.yaml
webhook:
  enabled: true
  ingress:
    enabled: true
    className: nginx
    annotations:
      cert-manager.io/cluster-issuer: letsencrypt
    hosts:
      - host: discord-bot.example.com
        paths:
          - path: /
            pathType: Prefix
    tls:
      - secretName: discord-bot-tls
        hosts:
          - discord-bot.example.com
```

### アップグレード

```bash
helm upgrade role-panel-bot ./helm/role-panel-bot -f values.yaml
```

### アンデプロイ

```bash
helm uninstall role-panel-bot
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
