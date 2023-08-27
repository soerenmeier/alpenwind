Alpenwind consists of a core part which contains the login and the 
"app drawer".

This can be extended with apps consisting of a library and some js.

## Dev build

The first thing that needs to be done is create the configuration file.
`core/server/config.toml`

```toml
listen-on = "127.0.0.1:5701"

[database]
host = "localhost"
name = "alpenwind"
user = "alpenwind"
password = "alpenwind"

[apps]
files = [
	"../../cinema/server/target/debug/libcinema_server.so",
	"../../pwvault/server/target/debug/libpwvault_server.so"
]

[pwvault]
favicons-dir = "../../data/favicons"

[cinema]
movies-dir = "../../data/movies"
movie-posters-dir = "../../data/posters"
series-dir = "../../data/series"
scaled-movies-posters = "../../data/scaled_posters/movies"
scaled-series-posters = "../../data/scaled_posters/series"
allow-deletes = true
```

Then you can run it.
`cd core/server && cargo r`
`cd core/ui && npm run dev`