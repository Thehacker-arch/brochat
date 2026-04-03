<div align="center">

# Welcome to BroChat

</div>

<div align="center"> 

  <img src="./UI/frontend/src-tauri/icons/icon.png"
    alt="Void Welcome"
		width="290"
	 	height="290"
    />
</div>

BroChat is a privacy-respecting, minimalist real-time chat application built from scratch for niche businesses that need simple, secure, and effective communication. Just like Slack, but stripped to the essentials — public chat, DMs, no distractions.

Built with:

- **Rust + Axum** backend (fast, safe, async-ready)
- **Tauri + Vue.js** frontend (native desktop feel with web flexibility)
- **PostgreSQL** database (reliable, relational, robust)

## 🛠️ Tech Stack

| Layer         | Stack                         |
|---------------|-------------------------------|
| Backend       | [Rust](https://www.rust-lang.org/) + [Axum](https://docs.rs/axum) |
| Frontend      | [Tauri](https://tauri.app/) + [Vue.js 3](https://vuejs.org/) |
| Database      | [PostgreSQL](https://www.postgresql.org/) |
| Auth          | JWT                           |
| Protocol      | WebSockets (for real-time chat) |



## 🧠 Why BroChat?

Most chat platforms are bloated, full of unnecessary features and data mining. BroChat focuses on core communication:

- **Public Chat** — where every registered user can participate.
- **Direct Messages (DMs)** — for one-on-one convos with end-to-end routing.
- Built **from scratch**, not on any third-party chat API.
- Designed for **cross-platform deployment** via Tauri.
- Uses **JWT-based authentication** for secure login and user sessions.

Perfect for:

- Small teams
- Niche internal networks
- Companies wanting chat ownership (no third-party dependencies)


## 🚀 Features

- ✅ Auth system with login & registration
- ✅ Real-time Public Chat
- ✅ Real-time Direct Messaging (DMs)
- ✅ Avatar upload support (optional)
- ✅ Native app for all platforms (Tauri-powered)
- 🔒 Token-based auth (no cookie mess)
- 🔧 Fully customizable & open source

---

<div style="display: flex; justify-content: center; gap: 10px;">
<table align="center">
  <tr>
    <td>
      <img src="./UI/frontend/src-tauri/icons/photo.png" alt="Void Welcome" height="430" width="200"/>
    </td>
    <td>
      <img src="./UI/frontend/src-tauri/icons/photo2.png" alt="Void Welcome" height="430" width="200"/>
    </td>
  </tr>
</table>
</div

---

## Run Locally
> ⚠️ Requirements: `Rust`, `Node.js`, `PostgreSQL`, `Tauri CLI`

## Commands in backend folder

```sh
export DATABASE_URL=postgres://name:port@localhost/database_name
```
```sh
cargo sqlx prepare
```

## Command for android (in UI/frontend)

```sh
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/$(ls -1 $ANDROID_HOME/ndk)"
```
```sh
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
```


## Database commands
Open Database
```sh
psql -U name -d database_name
```

CREATE

```sh
psql -U name -d postgres -c "CREATE DATABASE database_name;"
```

DELETE

```sh
psql -U name -d postgres -c "DROP DATABASE database_name;"
```

Migrate

```sh 
sqlx migrate run
```

### 1. Backend (Rust + Axum)

```bash
cd brochat/backend
cargo run
```

### 2. Frontend (Rust + VueJs)
```
cd brochat/UI/frontend/
```
#### For Web
```
npm run dev
```

#### For android
```
cargo tauri android dev
```

#### For ios
```
cargo tauri ios dev
```

#### For Desktop
```
cargo tauri dev
```

## Deploy on AWS (Easiest: Single EC2 + Docker Compose)

### 1) Launch EC2
- Ubuntu 24.04 instance
- Open inbound ports: `22` (SSH), `80` (HTTP), `443` (HTTPS)

### 2) Install Docker on EC2
```bash
sudo apt-get update
sudo apt-get install -y docker.io docker-compose-v2
sudo usermod -aG docker $USER
newgrp docker
```

### 3) Copy project and start services
```bash
git clone <your-repo-url>
cd brochat
docker compose up --build -d
```

### 4) Run database migration once
Your backend expects the schema from `backend/migrations/20250502140530_init.sql`.
Copy and execute migration in the postgres container:
```bash
docker cp backend/migrations/20250502140530_init.sql $(docker compose ps -q db):/tmp/init.sql
docker compose exec db psql -U brochat -d brochat -f /tmp/init.sql
```

### 5) Production env values
- Set a strong `JWT_SECRET` in `docker-compose.yml`
- Set `ALLOWED_ORIGINS` to your domain(s)
- Optional: add TLS with Nginx + Certbot or place ALB/CloudFront in front
