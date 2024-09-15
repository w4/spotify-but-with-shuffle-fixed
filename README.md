# spotify-but-with-shuffle-fixed

Spotify's shuffling is broken for large playlists and results in poor track selection because of machine learning or
whatever. I'm not exactly sure what it is, it's just terrible. Anyway, `spotify-but-with-shuffle-fixed` does what it
says on the tin.

When you start listening to a playlist, your liked songs, or an album, with shuffling on, the application will load the
entire playlist/collection/album into memory and apply the very basic Fisher-Yates shuffle to it. Whenever a track
change is detected, the application pops one of these tracks off and queues it to play next.

#### usage

`cargo run`
