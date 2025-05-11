<div align="center">

# Welcome to BroChat

</div>

<div align="center"> 

  <img src="./UI/frontend/src-tauri/icons/icon.png"
    alt="Void Welcome"
		width="300"
	 	height="300"
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
