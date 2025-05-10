## Command in backend folder

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
