## Command in backend folder

```export DATABASE_URL=postgres://anshsingh:0612@localhost/database_chats```

```cargo sqlx prepare```

## Command for android (in UI/frontend)

```export ANDROID_HOME="$HOME/Library/Android/sdk"```

```export NDK_HOME="$ANDROID_HOME/ndk/$(ls -1 $ANDROID_HOME/ndk)"```

```export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"```


## Database commands
Open Database
```psql -U anshsingh -d database_chats```

CREATE

```psql -U anshsingh -d postgres -c "CREATE DATABASE database_chats;"```

DELETE

```psql -U anshsingh -d postgres -c "DROP DATABASE database_chats;"```

Migrate

```sqlx migrate run``` (to be run in backend folder as the other database commands)
